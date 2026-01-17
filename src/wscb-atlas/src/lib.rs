use ::std::num::NonZeroU32;

use ::sdl3_sys::{pixels::SDL_PixelFormat, render::SDL_TextureAccess};
use ::wscb_sdl::copy_pixels;
use ::wscb_sdl::graph::Renderer;
use ::wscb_sdl::graph::{Surface, Texture};
use ::wscb_type::error::SdlError;
use ::wscb_type::graph::{Point, PointUnit, Rect, Size};
use sdl3_sys::rect::SDL_FRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle {
    pub(crate) index: NonZeroU32,
    pub rect: Rect,
}

#[derive(Debug)]
enum AtlasSegment {
    Static(Texture),
    Dynamic {
        texture: Texture,
        current_position: Point,
        current_line_height: PointUnit,
    },
}

impl AtlasSegment {
    #[must_use]
    pub fn allocate(&mut self, request: Size) -> Option<Point> {
        match self {
            AtlasSegment::Static(_) => return None,
            AtlasSegment::Dynamic {
                texture,
                current_position,
                current_line_height,
            } => {
                let size = texture.size().ok()?;
                let surface_height = size.height as i32;
                let surface_width = size.width as i32;

                if request.height > surface_height || request.width > surface_width {
                    return None;
                }

                // replace the current with new one once we alloc successfully
                let mut new_pen = *current_position;
                let mut new_line_height = *current_line_height;

                // try current line
                if new_pen.y.saturating_add(request.height) > surface_height {
                    // we can never satisfy this request
                    return None;
                }

                if new_pen.x.saturating_add(request.width) <= surface_width {
                    // current line is enough
                    let allocated = new_pen;

                    new_line_height = new_line_height.max(request.height);
                    new_pen.x = new_pen.x.saturating_add(request.width);

                    *current_position = new_pen;
                    *current_line_height = new_line_height;

                    return Some(allocated);
                }

                // try next line
                new_pen.y = new_pen.y.saturating_add(new_line_height);
                new_pen.x = 0;
                new_line_height = request.height;

                if new_pen.y.saturating_add(request.height) > surface_height {
                    // we can never satisfy this request
                    return None;
                }

                // we can hold it
                // width check is done in the first
                let allocated = new_pen;

                new_pen.x = new_pen.x.saturating_add(request.width);

                *current_position = new_pen;
                *current_line_height = new_line_height;

                return Some(allocated);
            }
        }
    }

    #[must_use]
    pub fn allocate_with_padding(&mut self, request: Size, padding: PointUnit) -> Option<Point> {
        let request = request.outset(padding);

        let mut allocated = self.allocate(request)?;
        allocated.x = allocated.x.saturating_add(padding);
        allocated.y = allocated.y.saturating_add(padding);

        Some(allocated)
    }
}

pub struct AtlasManager {
    segments: Vec<AtlasSegment>,
    current_index: usize,
    padding: PointUnit,
    default_size: Size,
    pixel_format: SDL_PixelFormat,
}

impl AtlasManager {
    pub fn empty(
        mut renderer: &mut Renderer,
        padding: PointUnit,
        atlas_segment_size: Size,
        pixel_format: SDL_PixelFormat,
    ) -> Result<Self, SdlError> {
        let mut this = Self {
            segments: Vec::new(),
            padding,
            default_size: atlas_segment_size,
            current_index: 0,
            pixel_format,
        };

        let allocated = this.alloc_segment(&mut renderer, None)?;
        this.segments.push(allocated);

        Ok(this)
    }

    fn alloc_segment(
        &mut self,
        renderer: &mut Renderer,
        minimum_size: Option<Size>,
    ) -> Result<AtlasSegment, SdlError> {
        let mut size = self.default_size;

        if let Some(minimum_size) = minimum_size {
            size = size.max_dimension(minimum_size);
        }

        let segment = AtlasSegment::Dynamic {
            texture: renderer.create_texture(
                self.pixel_format,
                SDL_TextureAccess::STREAMING,
                size.width as u32,
                size.height as u32,
            )?,
            current_position: Point::new(0, 0),
            current_line_height: 0,
        };

        Ok(segment)
    }

    pub fn allocate(
        &mut self,
        renderer: &mut Renderer,
        request: Size,
    ) -> Result<TextureHandle, SdlError> {
        let index = self.current_index;

        let segment = self.segments.get_mut(index).unwrap();
        let allocated = segment.allocate_with_padding(request, self.padding);

        if let Some(allocated) = allocated {
            return Ok(unsafe {
                TextureHandle {
                    index: NonZeroU32::new_unchecked((index.strict_add(1)) as u32),
                    rect: (allocated, request).into(),
                }
            });
        }

        // we need to create a new segment
        unsafe {
            let mut segment = self.alloc_segment(renderer, Some(request.outset(self.padding)))?;

            let index = NonZeroU32::new_unchecked(self.current_index.strict_add(1) as u32);

            let handle = TextureHandle {
                index,
                rect: (
                    segment
                        .allocate_with_padding(request, self.padding)
                        .expect("this allocation should never fail"),
                    request,
                )
                    .into(),
            };

            self.segments.push(segment);
            self.current_index = self.segments.len() - 1;

            return Ok(handle);
        }
    }

    pub fn allocate_then_copy_surface(
        &mut self,
        renderer: &mut Renderer,
        source: &Surface,
        source_rect: Option<Rect>,
    ) -> Result<TextureHandle, SdlError> {
        let src_format = source.format();
        if src_format != self.pixel_format {
            return Err(SdlError::sdl_err(
                "source surface format mismatch with atlas pixel format",
            ));
        }

        let source_size = source.size()?;
        let source_rect = source_rect.unwrap_or((Point::new(0, 0), source_size).into());

        let handle = self.allocate(renderer, source_rect.size)?;

        let dst_texture = self.get_texture(&handle);

        let src_pixels = source.pixels();
        let src_pitch = source.pitch();

        let guard = dst_texture.lock(handle.rect)?;

        let dst_pixels = guard.pixels;
        let dst_pitch = guard.pitch;

        copy_pixels(
            src_pixels,
            source_rect,
            src_pitch as isize,
            dst_pixels,
            Point::new(0, 0), // the `dst_pixel` has been offset by SDL itself
            dst_pitch as isize,
            self.pixel_format,
        )?;

        Ok(handle)
    }

    pub fn allocate_then_copy_surfaces(
        &mut self,
        renderer: &mut Renderer,
        sources: &[Surface],
    ) -> Vec<Result<TextureHandle, SdlError>> {
        let mut results: Vec<Result<TextureHandle, SdlError>> = (0..sources.len())
            .map(|_| Err(SdlError::SdlError("pending".to_string())))
            .collect();
        let mut segment_groups: Vec<Vec<(usize, TextureHandle)>> =
            vec![Vec::new(); self.segments.len()];

        for (i, source) in sources.iter().enumerate() {
            if source.format() != self.pixel_format {
                results[i] = Err(SdlError::sdl_err(
                    "source surface format mismatch with atlas pixel format",
                ));
                continue;
            }

            let source_size = match source.size() {
                Ok(s) => s,
                Err(e) => {
                    results[i] = Err(e);
                    continue;
                }
            };

            match self.allocate(renderer, source_size) {
                Ok(handle) => {
                    let segment_idx = (handle.index.get() - 1) as usize;
                    if segment_idx >= segment_groups.len() {
                        segment_groups.resize_with(segment_idx + 1, Vec::new);
                    }
                    segment_groups[segment_idx].push((i, handle));
                    results[i] = Ok(handle);
                }
                Err(e) => {
                    results[i] = Err(e);
                }
            }
        }

        for (segment_idx, group) in segment_groups.into_iter().enumerate() {
            if group.is_empty() {
                continue;
            }

            let texture = self.get_texture_from_index(segment_idx);

            // Calculate bounding box
            let mut min_x = PointUnit::MAX;
            let mut min_y = PointUnit::MAX;
            let mut max_x = 0;
            let mut max_y = 0;

            for (_, handle) in &group {
                min_x = min_x.min(handle.rect.position.x);
                min_y = min_y.min(handle.rect.position.y);
                max_x = max_x.max(handle.rect.position.x + handle.rect.size.width);
                max_y = max_y.max(handle.rect.position.y + handle.rect.size.height);
            }

            let bbox = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);

            // Lock once per segment
            match texture.lock(bbox) {
                Ok(guard) => {
                    for (orig_idx, handle) in group {
                        let source = &sources[orig_idx];
                        let src_pixels = source.pixels();
                        let src_pitch = source.pitch();

                        let dst_pixels = guard.pixels;
                        let dst_pitch = guard.pitch;

                        // Calculate dst_pos relative to bbox
                        let dst_pos = Point::new(
                            handle.rect.position.x - bbox.position.x,
                            handle.rect.position.y - bbox.position.y,
                        );

                        if let Err(e) = copy_pixels(
                            src_pixels,
                            (Point::new(0, 0), handle.rect.size).into(),
                            src_pitch as isize,
                            dst_pixels,
                            dst_pos,
                            dst_pitch as isize,
                            self.pixel_format,
                        ) {
                            results[orig_idx] = Err(e);
                        }
                    }
                }
                Err(e) => {
                    // If lock fails, all handles in this group fail
                    for (orig_idx, _) in group {
                        results[orig_idx] = Err(e.clone());
                    }
                }
            }
        }

        results
    }

    pub(crate) fn get_texture<'s>(&'s self, handle: &TextureHandle) -> &'s Texture {
        self.get_texture_from_index((handle.index.get() - 1) as usize)
    }

    pub(crate) fn get_texture_from_index<'s>(&'s self, idx: usize) -> &'s Texture {
        match &self.segments[idx] {
            AtlasSegment::Dynamic { texture, .. } => texture,
            AtlasSegment::Static(texture) => texture,
        }
    }

    pub(crate) fn get_texture_segment<'s>(&'s self, handle: &TextureHandle) -> &'s AtlasSegment {
        let idx = (handle.index.get() - 1) as usize;
        &self.segments[idx]
    }

    pub fn render(
        &self,
        renderer: &mut Renderer,
        handle: TextureHandle,
        dst: Option<Rect>,
    ) -> Result<(), SdlError> {
        let texture = self.get_texture(&handle);

        let src: SDL_FRect = handle.rect.into();
        let dst: Option<SDL_FRect> = dst.map(|r| r.into());

        unsafe {
            sdl3_sys::render::SDL_RenderTexture(
                renderer.get_pointer(),
                texture.get_pointer(),
                &src,
                dst.map(|f| &f as *const SDL_FRect)
                    .unwrap_or(std::ptr::null()),
            );
        }
        Ok(())
    }
}
