use image::DynamicImage;
use image::imageops::FilterType;

/// Encode an square image as rgb565 with an 8 bit alpha channel
pub fn encode_image(image: DynamicImage, nearest: bool) -> Option<Vec<u8>> {
    let buf = image
        .resize_to_fill(
            110,
            110,
            if nearest {
                FilterType::Nearest
            } else {
                FilterType::Gaussian
            },
        )
        .to_rgba8()
        .pixels()
        .flat_map(|p| {
            let [mut r, mut g, mut b, a] = p.0;

            // Mix alpha values against black
            let a = a as f64 / 255.0;
            r = (r as f64 * a) as u8;
            g = (g as f64 * a) as u8;
            b = (b as f64 * a) as u8;

            // Convert into rgb565 pixel type
            let [x, y] = rgb565::Rgb565::from_rgb888_components(r, g, b).to_rgb565_be();

            // Extend with hard coded alpha channel
            [x, y, 0xff]
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(buf.len(), 110 * 110 * 3);
    Some(buf)
}
