use ::std::{ops::Deref, ptr::NonNull};

use ::wscb_type::{
    error::SdlError,
    graph::Rect,
};

use crate::texture::Texture;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Renderer {
    pointer: NonNull<sdl3_sys::render::SDL_Renderer>,
}

impl Renderer {
    pub unsafe fn from_raw(raw: *mut sdl3_sys::render::SDL_Renderer) -> Option<Self> {
        Some(Self {
            pointer: NonNull::new(raw)?,
        })
    }

    pub fn get_pointer(&self) -> *mut sdl3_sys::render::SDL_Renderer {
        self.pointer.as_ptr()
    }

    pub fn create_texture(
        &self,
        format: sdl3_sys::pixels::SDL_PixelFormat,
        access: sdl3_sys::render::SDL_TextureAccess,
        w: u32,
        h: u32,
    ) -> Result<Texture, SdlError> {
        unsafe {
            let texture = sdl3_sys::render::SDL_CreateTexture(
                self.get_pointer(),
                format,
                access,
                w as i32,
                h as i32,
            );
            Texture::from_raw(texture).ok_or_else(|| SdlError::sdl_err("failed to create texture"))
        }
    }

    pub fn clear(&self) -> Result<(), SdlError> {
        unsafe {
            if !sdl3_sys::render::SDL_RenderClear(self.get_pointer()) {
                return Err(SdlError::sdl_err("failed to clear renderer"));
            }
        }
        Ok(())
    }

    pub fn present(&self) -> Result<(), SdlError> {
        unsafe {
            if !sdl3_sys::render::SDL_RenderPresent(self.get_pointer()) {
                return Err(SdlError::sdl_err("failed to present renderer"));
            }
        }
        Ok(())
    }

    pub fn copy_texture(&self, texture: &Texture, src_rect: Option<&Rect>, dst_rect: Option<&Rect>) -> Result<(), SdlError> {
        unsafe {
            let src_sdl_rect: Option<sdl3_sys::rect::SDL_FRect> = src_rect.map(|r| (*r).into());
            let dst_sdl_rect: Option<sdl3_sys::rect::SDL_FRect> = dst_rect.map(|r| (*r).into());

            let src_ptr = src_sdl_rect.as_ref().map(|r| r as *const _).unwrap_or(std::ptr::null());
            let dst_ptr = dst_sdl_rect.as_ref().map(|r| r as *const _).unwrap_or(std::ptr::null());

            if !sdl3_sys::render::SDL_RenderTexture(
                self.get_pointer(),
                texture.get_pointer(),
                src_ptr,
                dst_ptr,
            ) {
                return Err(SdlError::sdl_err("failed to copy texture"));
            }
        }
        Ok(())
    }

    pub fn set_draw_color(&self, r: u8, g: u8, b: u8, a: u8) -> Result<(), SdlError> {
        unsafe {
            if !sdl3_sys::render::SDL_SetRenderDrawColor(self.get_pointer(), r, g, b, a) {
                return Err(SdlError::sdl_err("failed to set draw color"));
            }
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            sdl3_sys::render::SDL_DestroyRenderer(self.pointer.as_ptr());
        }
    }
}

impl Deref for Renderer {
    type Target = sdl3_sys::render::SDL_Renderer;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.get_pointer() }
    }
}
