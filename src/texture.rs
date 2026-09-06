pub const FACE_SIZE: u32 = 256;
pub const ATLAS_COLS: u32 = 3;
pub const ATLAS_ROWS: u32 = 2;
const BYTES_PER_PIXEL: u32 = 4;

const FACE_0: &[u8] = include_bytes!("../assets/face_0.bmp");
const FACE_1: &[u8] = include_bytes!("../assets/face_1.bmp");
const FACE_2: &[u8] = include_bytes!("../assets/face_2.bmp");
const FACE_3: &[u8] = include_bytes!("../assets/face_3.bmp");
const FACE_4: &[u8] = include_bytes!("../assets/face_4.bmp");
const FACE_5: &[u8] = include_bytes!("../assets/face_5.bmp");

const FACES: [&[u8]; 6] = [FACE_0, FACE_1, FACE_2, FACE_3, FACE_4, FACE_5];

fn parse_bmp_rgba(data: &[u8]) -> (u32, u32, Vec<u8>) {
    assert!(data.len() >= 54, "BMP file too small");
    assert_eq!(&data[0..2], b"BM", "Not a BMP file");

    let dib_size = u32::from_le_bytes([data[14], data[15], data[16], data[17]]);
    assert_eq!(
        dib_size, 40,
        "Unsupported DIB header (only BITMAPINFOHEADER)"
    );

    let pixel_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
    let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]) as u32;
    let height = i32::from_le_bytes([data[22], data[23], data[24], data[25]]) as u32;
    let bit_count = u16::from_le_bytes([data[28], data[29]]);
    let compression = u32::from_le_bytes([data[30], data[31], data[32], data[33]]);

    assert_eq!(bit_count, 32, "Only 32-bit BMP supported");
    assert_eq!(compression, 0, "Only uncompressed BMP (BI_RGB) supported");

    let row_size = ((width * 32 + 31) / 32) * 4;
    let pixel_data = &data[pixel_offset..];

    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        let src_row = (height - 1 - y) * row_size;
        let dst_row = y * width * 4;
        for x in 0..width {
            let src = (src_row + x * 4) as usize;
            let dst = (dst_row + x * 4) as usize;
            rgba[dst] = pixel_data[src + 2];
            rgba[dst + 1] = pixel_data[src + 1];
            rgba[dst + 2] = pixel_data[src];
            rgba[dst + 3] = pixel_data[src + 3];
        }
    }

    (width, height, rgba)
}

pub fn create_texture_atlas() -> (Vec<u8>, u32, u32) {
    let atlas_width = FACE_SIZE * ATLAS_COLS;
    let atlas_height = FACE_SIZE * ATLAS_ROWS;
    let mut atlas_pixels = vec![0u8; (atlas_width * atlas_height * BYTES_PER_PIXEL) as usize];

    for (fi, face_bmp) in FACES.iter().enumerate() {
        let (w, h, face_rgba) = parse_bmp_rgba(face_bmp);
        assert_eq!(w, FACE_SIZE, "face_{}.bmp width must be {}", fi, FACE_SIZE);
        assert_eq!(h, FACE_SIZE, "face_{}.bmp height must be {}", fi, FACE_SIZE);

        let col = (fi % ATLAS_COLS as usize) as u32;
        let row = (fi / ATLAS_COLS as usize) as u32;

        for y in 0..FACE_SIZE {
            for x in 0..FACE_SIZE {
                let src_idx = ((y * FACE_SIZE + x) * BYTES_PER_PIXEL) as usize;
                let dst_x = col * FACE_SIZE + x;
                let dst_y = row * FACE_SIZE + y;
                let dst_idx = ((dst_y * atlas_width + dst_x) * BYTES_PER_PIXEL) as usize;
                atlas_pixels[dst_idx..dst_idx + BYTES_PER_PIXEL as usize]
                    .copy_from_slice(&face_rgba[src_idx..src_idx + BYTES_PER_PIXEL as usize]);
            }
        }
    }

    (atlas_pixels, atlas_width, atlas_height)
}
