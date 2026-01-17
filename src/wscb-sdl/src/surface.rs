use ::std::{ops::Deref, ptr::NonNull};

use ::wscb_type::{
    error::SdlError,
    graph::Size,
};

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
            Ok(Size::new((*surface).w, (*surface).h))
        }
    }

    pub fn format(&self) -> sdl3_sys::pixels::SDL_PixelFormat {
        unsafe { (*self.get_pointer()).format }
    }

    pub fn pixels(&self) -> *mut u8 {
        unsafe { (*self.get_pointer()).pixels as *mut u8 }
    }

    pub fn pitch(&self) -> i32 {
        unsafe { (*self.get_pointer()).pitch }
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
