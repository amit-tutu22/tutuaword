use tw_model::ImageData;

/// Re-encode raster image bytes as JPEG at the given quality (1–100).
pub fn compress_image_data(data: &ImageData, quality: u8) -> Result<ImageData, String> {
    use image::codecs::jpeg::JpegEncoder;
    use image::GenericImageView;
    use image::ImageReader;
    use std::io::Cursor;

    let quality = quality.clamp(1, 100);
    let reader = ImageReader::new(Cursor::new(&data.bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let img = reader.decode().map_err(|e| e.to_string())?;
    let (width_px, height_px) = img.dimensions();
    let rgb = img.to_rgb8();
    let mut out = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, quality);
    encoder
        .encode(
            rgb.as_raw(),
            width_px,
            height_px,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;
    Ok(ImageData {
        asset_id: uuid::Uuid::new_v4().to_string(),
        mime_type: "image/jpeg".into(),
        width_px,
        height_px,
        bytes: out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::ImageData;

    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
        0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn compress_png_to_jpeg() {
        let data = ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into()));
        let result = compress_image_data(&data, 80);
        assert!(result.is_ok(), "compress failed: {:?}", result.err());
        let jpeg = result.unwrap();
        assert_eq!(jpeg.mime_type, "image/jpeg");
        assert!(!jpeg.bytes.is_empty());
    }
}
