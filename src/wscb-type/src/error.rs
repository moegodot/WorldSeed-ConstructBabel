use ::std::num::TryFromIntError;

use ::thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SdlError {
    #[error("SDL error: {0}")]
    SdlError(String),
    #[error("try from int error when convert sdl integer to our integer: {0}")]
    TryFromIntError(#[from] TryFromIntError),
}

impl SdlError {
    pub fn check_sdl_error() -> Result<(), Self> {
        let error = sdl3_sys::error::SDL_GetError();

        if error.is_null() {
            return Ok(());
        }

        let mut e = unsafe { std::ffi::CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned();

        if !sdl3_sys::error::SDL_ClearError() {
            e.push_str("# failed to clear SDL error #");
        }

        Err(Self::SdlError(e))
    }

    pub fn sdl_err(msg: &str) -> Self {
        if let Err(e) = Self::check_sdl_error() {
            Self::SdlError(format!("(get sdl error {:?}) {}", e, msg))
        } else {
            Self::SdlError(format!("(no sdl error) {}", msg))
        }
    }
}
