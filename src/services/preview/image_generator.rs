use std::sync::Arc;

use ratatui::text::{Line, Span};
use tokio::sync::Mutex;

use super::PreviewContent;
use crate::{services::preview::GLOBAL_PICKER, utils::FileItem};

use super::PreviewGeneratorTrait;

/// Image preview generator
pub struct ImagePreviewGenerator;

impl PreviewGeneratorTrait for ImagePreviewGenerator {
    fn can_handle(&self, file: &FileItem) -> bool {
        file.is_image()
    }

    async fn generate_preview(&self, file: &FileItem, theme: &crate::theme::Theme) -> (String, PreviewContent) {
        let title = format!("🖼️ {}", file.name);

        // Try to load the image
        match image::open(&file.path) {
            Ok(img) => {
                // Create a protocol for the image
                let protocol = GLOBAL_PICKER.new_resize_protocol(img);

                (title, PreviewContent::image(Arc::new(Mutex::new(protocol))))
            }
            Err(e) => {
                let content = vec![
                    Line::from(vec![Span::styled(
                        "Image Load Error".to_string(),
                        theme.preview_error_style,
                    )]),
                    Line::from(vec![Span::raw("".to_string())]),
                    Line::from(vec![Span::styled(
                        format!("Failed to load image: {e}"),
                        theme.preview_info_style,
                    )]),
                ];
                (title, PreviewContent::text(content))
            }
        }
    }
}
