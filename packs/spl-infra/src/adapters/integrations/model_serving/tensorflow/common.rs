use bytes::Bytes;
use spl_domain::ports::integrations::PredictionResult;
use spl_shared::error::{AppError, Result};

/// Preprocessed image data ready for model inference
pub struct PreprocessedImage {
    pub data: Vec<Vec<Vec<f32>>>,   // Normalized tensor [H, W, C]
    pub resized_image_bytes: Bytes, // Original resized image
}

/// Mask data with binary mask and confidence
pub struct MaskData {
    pub binary_mask: Vec<u8>,
    pub confidence: f32,
}

/// Preprocesses image bytes to normalized tensor format
pub fn preprocess_image_to_tensor(image_bytes: &[u8], size: &u32) -> Result<PreprocessedImage> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::ValidationError(format!("Invalid or corrupted image: {}", e)))?;

    let resized = img.resize_exact(*size, *size, image::imageops::FilterType::Lanczos3);

    let rgb_image = resized.to_rgb8();
    let resized_bytes = Bytes::from(rgb_image.as_raw().clone());

    let mut tensor = vec![vec![vec![0.0f32; 3]; *size as usize]; *size as usize];
    for y in 0..*size {
        for x in 0..*size {
            let pixel = rgb_image.get_pixel(x, y);
            tensor[y as usize][x as usize][0] = pixel[0] as f32 / 255.0;
            tensor[y as usize][x as usize][1] = pixel[1] as f32 / 255.0;
            tensor[y as usize][x as usize][2] = pixel[2] as f32 / 255.0;
        }
    }

    Ok(PreprocessedImage {
        data: tensor,
        resized_image_bytes: resized_bytes,
    })
}

/// Extracts binary mask and confidence from model output
pub fn extract_mask_data(output: &[Vec<Vec<f32>>]) -> Result<MaskData> {
    let mut binary_mask = Vec::new();
    let mut prob_sum = 0.0;
    let mut above_threshold_count = 0;
    let threshold = 0.5;

    for row in output {
        for col in row {
            let prob = col.iter().copied().fold(f32::NEG_INFINITY, f32::max);

            if prob > threshold {
                binary_mask.push(255);
                prob_sum += prob;
                above_threshold_count += 1;
            } else {
                binary_mask.push(0);
            }
        }
    }

    let confidence = if above_threshold_count > 0 {
        prob_sum / above_threshold_count as f32
    } else {
        0.0
    };

    Ok(MaskData {
        binary_mask,
        confidence,
    })
}

/// Calculates disease severity as percentage of lesion overlap on leaf
pub fn calculate_severity(leaf: &MaskData, lesion: &MaskData) -> f32 {
    let mut overlap_count = 0;
    let mut leaf_count = 0;

    for i in 0..leaf.binary_mask.len().min(lesion.binary_mask.len()) {
        if leaf.binary_mask[i] == 255 {
            leaf_count += 1;
            if lesion.binary_mask[i] == 255 {
                overlap_count += 1;
            }
        }
    }

    if leaf_count > 0 {
        (overlap_count as f32 / leaf_count as f32) * 100.0
    } else {
        0.0
    }
}

/// Encodes image data to JPEG format
pub fn encode_to_jpeg(
    width: &u32,
    height: &u32,
    data: &[u8],
    color_type: image::ColorType,
) -> Result<Bytes> {
    let mut buffer = std::io::Cursor::new(Vec::new());

    image::write_buffer_with_format(
        &mut buffer,
        data,
        *width,
        *height,
        color_type,
        image::ImageFormat::Jpeg,
    )
    .map_err(|e| AppError::IntegrationError {
        integration: "tensorflow_serving".to_string(),
        message: format!("Failed to encode JPEG: {}", e),
    })?;

    Ok(Bytes::from(buffer.into_inner()))
}

/// Converts outputs to PredictionResult
pub fn build_prediction_result(
    output_0: &[Vec<Vec<f32>>],
    output_1: &[Vec<Vec<f32>>],
    resized_image_bytes: &Bytes,
    size: &u32,
) -> Result<PredictionResult> {
    let leaf_data = extract_mask_data(output_0)?;
    let lesion_data = extract_mask_data(output_1)?;
    let severity = calculate_severity(&leaf_data, &lesion_data);

    let encoded_image = encode_to_jpeg(size, size, resized_image_bytes, image::ColorType::Rgb8)?;
    let encoded_leaf_mask = encode_to_jpeg(size, size, &leaf_data.binary_mask, image::ColorType::L8)?;
    let encoded_lesion_mask = encode_to_jpeg(size, size, &lesion_data.binary_mask, image::ColorType::L8)?;

    Ok(PredictionResult {
        image: encoded_image,
        leaf_mask: encoded_leaf_mask,
        lesion_mask: encoded_lesion_mask,
        leaf_confidence: leaf_data.confidence,
        lesion_confidence: lesion_data.confidence,
        severity,
    })
}
