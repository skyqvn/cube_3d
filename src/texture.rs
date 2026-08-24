pub const FACE_SIZE: u32 = 256;
pub const ATLAS_COLS: u32 = 3;
pub const ATLAS_ROWS: u32 = 2;
const BYTES_PER_PIXEL: u32 = 4;
const BORDER_WIDTH: u32 = 4;
const CHAR_SIZE: u32 = 8;
const FONT_OFFSET: usize = 32;
const FONT_LAST: usize = 126;

pub const FACE_COLORS: [[u8; 4]; 6] = [
    [220, 60, 60, 255],  // 前 - 红
    [60, 180, 60, 255],  // 后 - 绿
    [60, 60, 220, 255],  // 右 - 蓝
    [220, 180, 40, 255], // 左 - 黄
    [200, 60, 200, 255], // 上 - 紫
    [60, 200, 200, 255], // 下 - 青
];

pub const FACE_LABELS: [&str; 6] = ["Front", "Back", "Right", "Left", "Top", "Bottom"];

pub fn create_face_texture(color: [u8; 4], label: &str) -> Vec<u8> {
    let size = FACE_SIZE;
    let mut pixels = vec![0u8; (size * size * BYTES_PER_PIXEL) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * BYTES_PER_PIXEL) as usize;
            if x < BORDER_WIDTH
                || x >= size - BORDER_WIDTH
                || y < BORDER_WIDTH
                || y >= size - BORDER_WIDTH
            {
                pixels[idx] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = 255;
            } else {
                pixels[idx] = color[0];
                pixels[idx + 1] = color[1];
                pixels[idx + 2] = color[2];
                pixels[idx + 3] = color[3];
            }
        }
    }

    // 使用 8×8 点阵字体绘制居中文字
    let font_bytes = include_bytes!("../assets/font_8x8.bin");
    let font_8x8: &[[u8; 8]; 96] = bytemuck::cast_slice(font_bytes).try_into().unwrap();

    let text_color = [255u8, 255, 255, 255];
    let start_x = (size - label.len() as u32 * CHAR_SIZE) / 2;
    let start_y = (size - CHAR_SIZE) / 2;

    for (ci, ch) in label.chars().enumerate() {
        let idx = ch as usize;
        if idx < FONT_OFFSET || idx > FONT_LAST {
            continue;
        }
        let bitmap = &font_8x8[idx - FONT_OFFSET];
        let cx = start_x + ci as u32 * CHAR_SIZE;
        for row in 0..CHAR_SIZE {
            let byte = bitmap[row as usize];
            for col in 0..CHAR_SIZE {
                if (byte >> (CHAR_SIZE - 1 - col)) & 1 == 1 {
                    let px = cx + col;
                    let py = start_y + row;
                    if px < size && py < size {
                        let pidx = ((py * size + px) * BYTES_PER_PIXEL) as usize;
                        pixels[pidx] = text_color[0];
                        pixels[pidx + 1] = text_color[1];
                        pixels[pidx + 2] = text_color[2];
                        pixels[pidx + 3] = text_color[3];
                    }
                }
            }
        }
    }

    pixels
}

pub fn create_texture_atlas() -> (Vec<u8>, u32, u32) {
    let atlas_width = FACE_SIZE * ATLAS_COLS;
    let atlas_height = FACE_SIZE * ATLAS_ROWS;
    let mut atlas_pixels = vec![0u8; (atlas_width * atlas_height * BYTES_PER_PIXEL) as usize];

    for (fi, color) in FACE_COLORS.iter().enumerate() {
        let col = (fi % ATLAS_COLS as usize) as u32;
        let row = (fi / ATLAS_COLS as usize) as u32;
        let face_data = create_face_texture(*color, FACE_LABELS[fi]);

        for y in 0..FACE_SIZE {
            for x in 0..FACE_SIZE {
                let src_idx = ((y * FACE_SIZE + x) * BYTES_PER_PIXEL) as usize;
                let dst_x = col * FACE_SIZE + x;
                let dst_y = row * FACE_SIZE + y;
                let dst_idx = ((dst_y * atlas_width + dst_x) * BYTES_PER_PIXEL) as usize;
                atlas_pixels[dst_idx..dst_idx + BYTES_PER_PIXEL as usize]
                    .copy_from_slice(&face_data[src_idx..src_idx + BYTES_PER_PIXEL as usize]);
            }
        }
    }

    (atlas_pixels, atlas_width, atlas_height)
}
