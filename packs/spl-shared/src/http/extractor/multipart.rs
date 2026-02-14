use crate::error::{AppError, Result};
use axum::extract::Multipart;
pub async fn extract_file(
    field_name: &str,
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, Option<String>)> {
    let mut file_bytes = None;
    let mut filename = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ValidationError(format!("Failed to process multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == field_name {
            if let Some(fname) = field.file_name() {
                filename = Some(fname.to_string());
            }

            let data = field.bytes().await.map_err(|e| {
                AppError::ValidationError(format!("Failed to read file bytes: {}", e))
            })?;
            file_bytes = Some(data.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError::ValidationError("No file provided".into()))?;

    Ok((bytes, filename))
}
