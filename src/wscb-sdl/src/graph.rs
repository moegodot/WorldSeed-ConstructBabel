use ::std::{ops::Deref, ptr::NonNull};

use ::wscb_type::{
    error::SdlError,
    graph::{Rect, Size},
};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Texture {
    pointer: NonNull<sdl3_sys::render::SDL_Texture>,
}

#[derive(Debug)]
pub struct LockedTextureGuard<'t> {
    texture: &'t Texture,
    pub pixels: *mut u8,
    pub pitch: usize,
}

impl Drop for LockedTextureGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            sdl3_sys::render::SDL_UnlockTexture(self.texture.get_pointer());
        }
    }
}

impl Texture {
    pub unsafe fn from_raw(raw: *mut sdl3_sys::render::SDL_Texture) -> Option<Self> {
        Some(Self {
            pointer: NonNull::new(raw)?,
        })
    }

    pub fn get_pointer(&self) -> *mut sdl3_sys::render::SDL_Texture {
        self.pointer.as_ptr()
    }

    pub fn size(&self) -> Result<Size, SdlError> {
        let mut w = 0.0;
        let mut h = 0.0;
        unsafe {
            if !sdl3_sys::render::SDL_GetTextureSize(self.get_pointer(), &mut w, &mut h) {
                return Err(SdlError::sdl_err("failed to get texture size"));
            }
        }
        Ok(Size::new(w as u32, h as u32))
    }

    pub fn format(&self) -> sdl3_sys::pixels::SDL_PixelFormat {
        unsafe {
            let props = sdl3_sys::render::SDL_GetTextureProperties(self.get_pointer());
            let format_val = sdl3_sys::properties::SDL_GetNumberProperty(
                props,
                sdl3_sys::render::SDL_PROP_TEXTURE_FORMAT_NUMBER,
                sdl3_sys::pixels::SDL_PixelFormat::UNKNOWN.0 as i64,
            );
            sdl3_sys::pixels::SDL_PixelFormat(format_val as i32)
        }
    }

    pub fn lock<'a>(&'a self, rect: Rect) -> Result<LockedTextureGuard<'a>, SdlError> {
        let mut pixels = std::ptr::null_mut();
        let mut pitch = 0;

        unsafe {
            if !sdl3_sys::render::SDL_LockTexture(
                self.get_pointer(),
                &rect.try_into()?,
                &mut pixels,
                &mut pitch,
            ) {
                return Err(SdlError::sdl_err("failed to lock texture"));
            }

            Ok(LockedTextureGuard {
                texture: self,
                pixels: pixels as *mut u8,
                pitch: pitch as usize,
            })
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            sdl3_sys::render::SDL_DestroyTexture(self.pointer.as_ptr());
        }
    }
}

impl Deref for Texture {
    type Target = sdl3_sys::render::SDL_Texture;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.get_pointer() }
    }
}

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
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            sdl3_sys::render::SDL_DestroyRenderer(self.pointer.as_ptr());
        }
    }
}
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Surface {
    pointer: NonNull<sdl3_sys::surface::SDL_Surface>,
}

impl Surface {
    pub unsafe fn from_raw(raw: *mut sdl3_sys::surface::SDL_Surface) -> Option<Self> {
        Some(Self {
            pointer: NonNull::new(raw)?,
        })
    }

    pub fn get_pointer(&self) -> *mut sdl3_sys::surface::SDL_Surface {
        self.pointer.as_ptr()
    }

    pub fn size(&self) -> Result<Size, SdlError> {
        unsafe {
            let surface = self.get_pointer();
            Ok(Size::new((*surface).w as u32, (*surface).h as u32))
        }
    }

    pub fn format(&self) -> sdl3_sys::pixels::SDL_PixelFormat {
        unsafe { (*self.get_pointer()).format }
    }

    pub fn pixels(&self) -> *mut u8 {
        unsafe { (*self.get_pointer()).pixels as *mut u8 }
    }

    pub fn pitch(&self) -> usize {
        unsafe { (*self.get_pointer()).pitch as usize }
    }

    pub fn duplicate(&self) -> Option<Self> {
        unsafe {
            Some(Self {
                pointer: NonNull::new(sdl3_sys::surface::SDL_DuplicateSurface(
                    self.pointer.as_ptr(),
                ))?,
            })
        }
    }
}

impl Deref for Surface {
    type Target = sdl3_sys::surface::SDL_Surface;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.get_pointer() }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            sdl3_sys::surface::SDL_DestroySurface(self.pointer.as_ptr());
        }
    }
}
