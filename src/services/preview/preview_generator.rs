use ratatui::text::{Line, Span};
use std::fs;

use super::PreviewContent;
use crate::utils::FileItem;

/// Trait for preview generators
pub trait PreviewGeneratorTrait {
    /// Generate preview content for a file
    #[allow(async_fn_in_trait)]
    async fn generate_preview(&self, file: &FileItem, theme: &crate::theme::Theme) -> (String, PreviewContent);

    /// Check if this generator can handle the given file
    fn can_handle(&self, file: &FileItem) -> bool;
}

use super::{
    DirectoryPreviewGenerator, ImagePreviewGenerator, PdfPreviewGenerator, TextPreviewGenerator,
};

/// Enum for different preview generators to support async trait methods
pub enum PreviewGeneratorType {
    Directory(DirectoryPreviewGenerator),
    Image(ImagePreviewGenerator),
    Pdf(PdfPreviewGenerator),
    Text(TextPreviewGenerator),
    Binary(BinaryPreviewGenerator),
}

impl PreviewGeneratorType {
    /// Check if this generator can handle the given file
    pub fn can_handle(&self, file: &FileItem) -> bool {
        match self {
            PreviewGeneratorType::Directory(generator) => generator.can_handle(file),
            PreviewGeneratorType::Image(generator) => generator.can_handle(file),
            PreviewGeneratorType::Pdf(generator) => generator.can_handle(file),
            PreviewGeneratorType::Text(generator) => generator.can_handle(file),
            PreviewGeneratorType::Binary(generator) => generator.can_handle(file),
        }
    }

    /// Generate preview content for a file
    pub async fn generate_preview(&self, file: &FileItem, theme: &crate::theme::Theme) -> (String, PreviewContent) {
        match self {
            PreviewGeneratorType::Directory(generator) => generator.generate_preview(file, theme).await,
            PreviewGeneratorType::Image(generator) => generator.generate_preview(file, theme).await,
            PreviewGeneratorType::Pdf(generator) => generator.generate_preview(file, theme).await,
            PreviewGeneratorType::Text(generator) => generator.generate_preview(file, theme).await,
            PreviewGeneratorType::Binary(generator) => generator.generate_preview(file, theme).await,
        }
    }
}

/// Main service for generating preview content for files and directories
pub struct PreviewGenerator;

impl PreviewGenerator {
    /// Generate preview content for a file or directory
    pub async fn generate_preview_content(file: &FileItem, theme: &crate::theme::Theme) -> (String, PreviewContent) {
        // Try different file preview generators in order
        let generators = vec![
            PreviewGeneratorType::Directory(DirectoryPreviewGenerator),
            PreviewGeneratorType::Image(ImagePreviewGenerator),
            PreviewGeneratorType::Pdf(PdfPreviewGenerator),
            PreviewGeneratorType::Text(TextPreviewGenerator),
        ];

        for generator in generators {
            if generator.can_handle(file) {
                return generator.generate_preview(file, theme).await;
            }
        }

        // Fallback to binary file preview
        let binary_gen = PreviewGeneratorType::Binary(BinaryPreviewGenerator);
        binary_gen.generate_preview(file, theme).await
    }
}

/// Process special characters in text for better display
pub fn process_special_characters(text: &str) -> String {
    let mut result = String::new();

    for ch in text.chars() {
        match ch {
            '\t' => {
                // Replace tab with visible representation and spaces
                result.push_str("→   "); // Arrow symbol followed by 3 spaces for tab width
            }
            '\r' => {
                // Replace carriage return with visible representation
                result.push_str("\\r");
            }
            '\0' => {
                // Replace null character with visible representation
                result.push_str("\\0");
            }
            c if c.is_control() && c != '\n' => {
                // Replace other control characters with their escape sequence
                result.push_str(&format!("\\x{:02x}", c as u8));
            }
            c => {
                // Keep normal characters as-is
                result.push(c);
            }
        }
    }

    result
}

/// Binary file preview generator (fallback)
pub struct BinaryPreviewGenerator;

impl PreviewGeneratorTrait for BinaryPreviewGenerator {
    fn can_handle(&self, _file: &FileItem) -> bool {
        // This is a fallback generator, so it can handle any file
        true
    }

    async fn generate_preview(&self, file: &FileItem, theme: &crate::theme::Theme) -> (String, PreviewContent) {
        let title = format!("📄 {}", file.name);

        // Get file metadata
        let metadata = match fs::metadata(&file.path) {
            Ok(metadata) => metadata,
            Err(e) => {
                let content = vec![Line::from(vec![Span::styled(
                    format!("Error reading file metadata: {e}"),
                    theme.preview_error_style,
                )])];
                return (title, PreviewContent::text(content));
            }
        };

        let file_size = metadata.len();

        let content = vec![
            Line::from(vec![Span::styled(
                "Binary File".to_string(),
                theme.preview_placeholder_style,
            )]),
            Line::from(vec![Span::raw("".to_string())]),
            Line::from(vec![Span::styled(
                format!("Size: {file_size} bytes"),
                theme.preview_info_style,
            )]),
            Line::from(vec![Span::styled(
                "Cannot preview binary content".to_string(),
                theme.preview_info_style,
            )]),
            Line::from(vec![Span::raw("".to_string())]),
            Line::from(vec![Span::styled(
                "File type: Binary/Unknown".to_string(),
                theme.dir_style,
            )]),
        ];

        (title, PreviewContent::text(content))
    }
}
