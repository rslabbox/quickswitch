use anyhow::{Ok, Result};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use ratatui::{
    text::Span,
};
use serde::{Deserialize, Serialize};
use std::{
    io::IsTerminal,
    path::{Path, PathBuf},
};
use tracing::{debug, error, instrument};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    Powershell,
    /// Command Prompt (Windows)
    Cmd,
}

pub fn is_tty() -> bool {
    std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && std::io::stderr().is_terminal()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AppMode {
    Normal,  // Default navigation mode (command mode)
    History, // History selection mode
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub path: PathBuf,
    pub frequency: u32,
    pub last_accessed: DateTime<Utc>,
    pub first_accessed: DateTime<Utc>,
}

impl HistoryEntry {
    pub fn new(path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            path,
            frequency: 1,
            last_accessed: now,
            first_accessed: now,
        }
    }

    pub fn increment_frequency(&mut self) {
        self.frequency += 1;
        self.last_accessed = Utc::now();
    }

    /// Calculate score for sorting (frequency with time decay)
    pub fn calculate_score(&self, time_decay_days: u32) -> f64 {
        let frequency_weight = self.frequency as f64;
        let time_decay = self.calculate_time_decay(time_decay_days);
        frequency_weight * time_decay
    }

    fn calculate_time_decay(&self, decay_days: u32) -> f64 {
        let days_since_access = (Utc::now() - self.last_accessed).num_days();
        if days_since_access <= 0 {
            1.0
        } else {
            let decay_factor = days_since_access as f64 / decay_days as f64;
            (1.0 - decay_factor.min(1.0)).max(0.1) // Minimum 10% weight
        }
    }
}

#[derive(Clone, Debug)]
pub enum HistorySortMode {
    Frequency,       // Sort by frequency only
    Recent,          // Sort by last accessed time
    FrequencyRecent, // Sort by frequency with time decay
    Alphabetical,    // Sort alphabetically
}

#[derive(Clone, Debug)]
pub enum DisplayItem {
    File(FileItem),
    History(HistoryEntry),
}

impl DisplayItem {
    pub fn get_display_name(&self) -> String {
        match self {
            DisplayItem::File(file) => file.name.clone(),
            DisplayItem::History(entry) => entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string(),
        }
    }

    pub fn get_path(&self) -> &PathBuf {
        match self {
            DisplayItem::File(file) => &file.path,
            DisplayItem::History(entry) => &entry.path,
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            DisplayItem::File(file) => file.is_dir,
            DisplayItem::History(entry) => entry.path.is_dir(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

impl FileItem {
    pub fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let is_dir = path.is_dir();
        Self {
            name,
            path: path.to_path_buf(),
            is_dir,
        }
    }

    /// Check if the file is an image based on its extension
    pub fn is_image(&self) -> bool {
        if self.is_dir {
            return false;
        }

        let extension = self
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        matches!(
            extension.as_deref(),
            Some("jpg")
                | Some("jpeg")
                | Some("png")
                | Some("gif")
                | Some("bmp")
                | Some("webp")
                | Some("tiff")
                | Some("tif")
                | Some("svg")
                | Some("ico")
                | Some("avif")
        )
    }

    pub fn is_pdf(&self) -> bool {
        if self.is_dir {
            return false;
        }

        let extension = self
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        matches!(extension.as_deref(), Some("pdf"))
    }
}

pub fn highlight_search_term<'a>(
    text: &'a str, 
    search: &'a str, 
    theme: &crate::theme::Theme,
    base_style: ratatui::style::Style,
) -> Vec<Span<'a>> {
    if search.is_empty() {
        return vec![Span::styled(text, base_style)];
    }

    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut spans = Vec::new();
    let mut last_end = 0;

    while let Some(start) = text_lower[last_end..].find(&search_lower) {
        let actual_start = last_end + start;
        let actual_end = actual_start + search.len();

        if actual_start > last_end {
            spans.push(Span::styled(&text[last_end..actual_start], base_style));
        }

        spans.push(Span::styled(
            &text[actual_start..actual_end],
            theme.search_match_style,
        ));

        last_end = actual_end;
    }

    if last_end < text.len() {
        spans.push(Span::styled(&text[last_end..], base_style));
    }

    spans
}

pub fn run_non_interactive() -> Result<()> {
    println!("{}", std::env::current_dir()?.display());
    Ok(())
}

// Init Bash and Zsh functions for quickswitch
#[instrument]
fn qs_init_bash_zsh() -> Result<()> {
    let bash_init = r#"
qs() {
    local dir
    dir=$(quickswitch 2>&1 >/dev/tty | tail -n 1)
    if [ -d "$dir" ]; then
        cd "$dir"
    fi
}

qshs() {
    local dir
    dir=$(quickswitch --mode history 2>&1 >/dev/tty | tail -n 1)
    if [ -d "$dir" ]; then
        cd "$dir"
    fi
}
    "#;
    println!("{bash_init}");
    debug!("{bash_init}");

    Ok(())
}

#[instrument]
fn qs_init_fish() -> Result<()> {
    let fish_init = r#"
function qs
    set -l result (quickswitch 2>&1 >/dev/tty)

    if [ -n "$result" ]
        cd -- $result

        # Remove last token from commandline.
        commandline -t ""
        commandline -it -- $prefix
    end

    commandline -f repaint
end

function qshs
    set -l result (quickswitch --mode history 2>&1 >/dev/tty)

    if [ -n "$result" ]
        cd -- $result

        # Remove last token from commandline.
        commandline -t ""
        commandline -it -- $prefix
    end

    commandline -f repaint
end
    "#;
    println!("{fish_init}");
    debug!("{fish_init}");

    Ok(())
}

#[instrument]
fn qs_init_powershell() -> Result<()> {
    let powershell_init = r#"
function qs {
    $errorFile = [System.IO.Path]::GetTempFileName()
    Start-Process -FilePath "quickswitch.exe" -NoNewWindow -Wait -RedirectStandardError $errorFile
    $errorOutput = Get-Content -Path $errorFile -Encoding UTF8
    Remove-Item $errorFile
    if ($errorOutput -and (Test-Path $errorOutput)) {
        cd $errorOutput
    }
}

function qshs {
    $errorFile = [System.IO.Path]::GetTempFileName()
    Start-Process -FilePath "quickswitch.exe" -NoNewWindow -Wait -RedirectStandardError $errorFile -ArgumentList "--mode history"
    $errorOutput = Get-Content -Path $errorFile -Encoding UTF8
    Remove-Item $errorFile
    if ($errorOutput -and (Test-Path $errorOutput)) {
        cd $errorOutput
    }
}
    "#;
    println!("{powershell_init}");
    debug!("{powershell_init}");

    Ok(())
}

#[instrument]
fn qs_init_cmd() -> Result<()> {
    error!("CMD initialization is not implemented yet. Please use PowerShell or another shell.");
    todo!("CMD initialization is not implemented yet");
}

pub fn qs_init(shell: ShellType) -> Result<()> {
    match shell {
        ShellType::Bash => qs_init_bash_zsh(),
        ShellType::Zsh => qs_init_bash_zsh(),
        ShellType::Fish => qs_init_fish(),
        ShellType::Powershell => qs_init_powershell(),
        ShellType::Cmd => qs_init_cmd(),
    }
}
