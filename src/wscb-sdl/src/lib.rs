use ::sdl3_sys::pixels::SDL_BITSPERPIXEL;
use ::wscb_type::{
    error::SdlError,
    graph::{Point, Rect},
};

pub mod graph;

pub fn copy_pixels(
    src: *const u8,
    src_rect: Rect,
    src_pitch: usize,
    dst: *mut u8,
    dst_pos: Point,
    dst_pitch: usize,
    pixel_format: sdl3_sys::pixels::SDL_PixelFormat,
) -> Result<(), SdlError> {
    let pixel_byte = match SDL_BITSPERPIXEL(pixel_format) {
        8 => 1,
        16 => 2,
        24 => 3,
        32 => 4,
        40 => 5,
        48 => 6,
        56 => 7,
        64 => 8,
        72 => 9,
        80 => 10,
        88 => 11,
        96 => 12,
        104 => 13,
        112 => 14,
        120 => 15,
        128 => 16,
        136 => 17,
        144 => 18,
        152 => 19,
        160 => 20,
        168 => 21,
        176 => 22,
        184 => 23,
        192 => 24,
        200 => 25,
        208 => 26,
        216 => 27,
        224 => 28,
        232 => 29,
        240 => 30,
        248 => 31,
        // 256 will overflow
        _ => {
            return Err(SdlError::sdl_err(
                "unsupported pixel format because of unsupported bitspixel",
            ));
        }
    };

    let dst_x_start: usize = dst_pos.x.try_into()?;
    let dst_y_start: usize = dst_pos.y.try_into()?;

    let src_x_start: usize = src_rect.position.x as usize;
    let src_y_start: usize = src_rect.position.y as usize;

    let mut y_index = 0usize;

    let src_height = src_rect.size.height.try_into()?;

    while y_index < src_height {
        unsafe {
            let src_line = src.add((src_y_start + y_index) * src_pitch);
            let dst_line = dst.add((dst_y_start + y_index) * dst_pitch);

            let src_pos = src_line.add(src_x_start * pixel_byte);
            let dst_pos = dst_line.add(dst_x_start * pixel_byte);

            std::ptr::copy_nonoverlapping(
                src_pos,
                dst_pos,
                src_rect.size.width as usize * pixel_byte,
            );
        }

        y_index += 1;
    }

    Ok(())
}
