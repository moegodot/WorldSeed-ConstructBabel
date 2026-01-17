use ::std::{ops::Deref, ptr::NonNull};
#[cfg(target_os = "macos")]
use std::ffi::c_void;

use ::wscb_type::{error::SdlError, graph::Size};
use raw_window_handle::{AppKitWindowHandle, HandleError, RawWindowHandle, WindowHandle};

use crate::renderer::Renderer;

#[derive(Debug, PartialEq, Eq)]
pub struct Window {
    pointer: NonNull<sdl3_sys::video::SDL_Window>,
    #[cfg(target_os = "macos")]
    ns_view: NonNull<c_void>,
}

impl Window {
    pub fn new(title: &str, width: i32, height: i32) -> Result<Self, SdlError> {
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let flags = sdl3_sys::video::SDL_WindowFlags::RESIZABLE | sdl3_sys::video::SDL_WindowFlags::METAL;
                } else {
                    let flags = sdl3_sys::video::SDL_WindowFlags::RESIZABLE;
                }
            }

            let c_title = std::ffi::CString::new(title)
                .map_err(|_| SdlError::sdl_err("invalid title string"))?;

            let pointer = sdl3_sys::video::SDL_CreateWindow(c_title.as_ptr(), width, height, flags);
            let pointer = NonNull::new(pointer)
                .ok_or_else(|| SdlError::sdl_err("failed to create window"))?;

            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let view = sdl3_sys::metal::SDL_Metal_CreateView(pointer.as_ptr());
                    let view = NonNull::new(view).ok_or_else(|| SdlError::sdl_err("failed to create NSView"))?;

                    Ok(Self { pointer: pointer, ns_view: view })
                }
                else{
                    unimplemented!()
                }
            }
        }
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

    pub fn title(&self) -> String {
        unsafe {
            let title_ptr = sdl3_sys::video::SDL_GetWindowTitle(self.get_pointer());
            if title_ptr.is_null() {
                return "".to_string();
            }
            std::ffi::CStr::from_ptr(title_ptr)
                .to_string_lossy()
                .to_string()
        }
    }

    pub fn set_title(&self, title: &str) -> Result<(), SdlError> {
        let c_title =
            std::ffi::CString::new(title).map_err(|_| SdlError::sdl_err("invalid title string"))?;
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
            let renderer =
                sdl3_sys::render::SDL_CreateRenderer(self.get_pointer(), std::ptr::null());
            Renderer::from_raw(renderer)
                .ok_or_else(|| SdlError::sdl_err("failed to create renderer"))
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    sdl3_sys::metal::SDL_Metal_DestroyView(self.ns_view.as_ptr());
                }
            }

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

impl raw_window_handle::HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                unimplemented!()
            } else if #[cfg(target_os = "macos")] {
                unsafe{
                    Ok(WindowHandle::borrow_raw(
                        RawWindowHandle::AppKit(AppKitWindowHandle::new(self.ns_view))
                    ))
                }
            } else {
                unimplemented!()
            }
        }
    }
}
