use ::std::{ops::Deref, ptr::NonNull};

use ::wscb_type::{
    error::SdlError,
    graph::Size,
};

use crate::renderer::Renderer;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Window {
    pointer: NonNull<sdl3_sys::video::SDL_Window>,
}

impl Window {
    pub unsafe fn from_raw(raw: *mut sdl3_sys::video::SDL_Window) -> Option<Self> {
        Some(Self {
            pointer: NonNull::new(raw)?,
        })
    }

    pub fn get_pointer(&self) -> *mut sdl3_sys::video::SDL_Window {
        self.pointer.as_ptr()
    }

    pub fn size(&self) -> Result<Size, SdlError> {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            if !sdl3_sys::video::SDL_GetWindowSize(self.get_pointer(), &mut w, &mut h) {
                return Err(SdlError::sdl_err("failed to get window size"));
            }
        }
        Ok(Size::new(w, h))
    }

    pub fn set_size(&self, size: Size) -> Result<(), SdlError> {
        unsafe {
            if !sdl3_sys::video::SDL_SetWindowSize(self.get_pointer(), size.width, size.height) {
                return Err(SdlError::sdl_err("failed to set window size"));
            }
        }
        Ok(())
    }

    pub fn title(&self) -> &str {
        unsafe {
            let title_ptr = sdl3_sys::video::SDL_GetWindowTitle(self.get_pointer());
            if title_ptr.is_null() {
                return "";
            }
            std::ffi::CStr::from_ptr(title_ptr)
                .to_str()
                .unwrap_or("")
        }
    }

    pub fn set_title(&self, title: &str) -> Result<(), SdlError> {
        let c_title = std::ffi::CString::new(title)
            .map_err(|_| SdlError::sdl_err("invalid title string"))?;
        unsafe {
            if !sdl3_sys::video::SDL_SetWindowTitle(self.get_pointer(), c_title.as_ptr()) {
                return Err(SdlError::sdl_err("failed to set window title"));
            }
        }
        Ok(())
    }

    pub fn show(&self) -> Result<(), SdlError> {
        unsafe {
            if !sdl3_sys::video::SDL_ShowWindow(self.get_pointer()) {
                return Err(SdlError::sdl_err("failed to show window"));
            }
        }
        Ok(())
    }

    pub fn hide(&self) -> Result<(), SdlError> {
        unsafe {
            if !sdl3_sys::video::SDL_HideWindow(self.get_pointer()) {
                return Err(SdlError::sdl_err("failed to hide window"));
            }
        }
        Ok(())
    }

    pub fn create_renderer(&self) -> Result<Renderer, SdlError> {
        unsafe {
            let renderer = sdl3_sys::render::SDL_CreateRenderer(self.get_pointer(), std::ptr::null());
            Renderer::from_raw(renderer).ok_or_else(|| SdlError::sdl_err("failed to create renderer"))
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            sdl3_sys::video::SDL_DestroyWindow(self.pointer.as_ptr());
        }
    }
}

impl Deref for Window {
    type Target = sdl3_sys::video::SDL_Window;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.get_pointer() }
    }
}
