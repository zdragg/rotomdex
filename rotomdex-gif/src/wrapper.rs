//!  Decoding of GIF Images
//!
//!  GIF (Graphics Interchange Format) is an image format that supports lossless compression.
//!
//!  # Related Links
//!  * <http://www.w3.org/Graphics/GIF/spec-gif89a.txt> - The GIF Specification
use core::ops::Range;

use alloc::vec::Vec;

use crate::DisposalMethod;
use crate::common::GifRead;
use crate::{ColorOutput, DecodingError};

// A set of pixels, with a defined delay.
pub struct RgbaFrame {
    pub buffer: RgbaImageBuffer,
    pub delay: u64,
}

// A set of pixels.
pub struct RgbaImageBuffer {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl RgbaImageBuffer {
    /// Constructs a buffer from a `Vec<u8>`
    ///
    /// Returns `None` if the buffer is not big enough (including when the image dimensions
    /// necessitate an allocation of more bytes than supported by the buffer).
    pub fn from_raw(width: u16, height: u16, buf: Vec<u8>) -> Option<Self> {
        if Self::check_image_fits(width, height, buf.len()) {
            Some(Self {
                data: buf,
                width,
                height,
            })
        } else {
            None
        }
    }

    /// Creates a new image buffer.
    ///
    /// all the pixels of this image have a value of zero.
    ///
    /// # Panics
    ///
    /// Panics when the resulting image is larger than the maximum size of a vector.
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        let size = Self::image_buffer_len(width, height)
            .expect("Buffer length in `RgbaImageBuffer::new` overflows usize");
        Self {
            data: vec![0; size],
            width,
            height,
        }
    }

    /// Gets a reference to the pixel at location `(x, y)`
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds `(width, height)`.
    #[inline]
    #[track_caller]
    pub fn get_pixel(&self, x: u16, y: u16) -> &[u8; 4] {
        match self.pixel_indices(x, y) {
            None => panic!(
                "Image index {:?} out of bounds {:?}",
                (x, y),
                (self.width, self.height)
            ),
            Some(pixel_indices) => (&self.data[pixel_indices]).try_into().unwrap(),
        }
    }

    /// Gets a reference to the mutable pixel at location `(x, y)`
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds `(width, height)`.
    #[inline]
    #[track_caller]
    pub fn get_pixel_mut(&mut self, x: u16, y: u16) -> &mut [u8; 4] {
        match self.pixel_indices(x, y) {
            None => panic!(
                "Image index {:?} out of bounds {:?}",
                (x, y),
                (self.width, self.height)
            ),
            Some(pixel_indices) => (&mut self.data[pixel_indices]).try_into().unwrap(),
        }
    }

    /// Enumerates over the pixels of the image.
    /// The iterator yields the coordinates of each pixel
    /// along with a mutable reference to them.
    pub fn enumerate_pixels_mut(&mut self) -> impl Iterator<Item = (u16, u16, &mut [u8; 4])> {
        let width = usize::from(self.width);
        self.pixels_mut()
            .enumerate()
            .map(move |(i, pixel)| ((i % width) as u16, (i / width) as u16, pixel))
    }

    /// Returns an iterator over the mutable pixels of this image.
    pub fn pixels_mut(&mut self) -> impl Iterator<Item = &mut [u8; 4]> {
        self.inner_pixels_mut().as_chunks_mut::<4>().0.iter_mut()
    }

    pub(crate) fn inner_pixels_mut(&mut self) -> &mut [u8] {
        let len = Self::image_buffer_len(self.width, self.height).unwrap();
        &mut self.data[..len]
    }

    /// Test that the image fits inside the buffer.
    fn check_image_fits(width: u16, height: u16, len: usize) -> bool {
        let checked_len = Self::image_buffer_len(width, height);
        checked_len.is_some_and(|min_len| min_len <= len)
    }

    fn image_buffer_len(width: u16, height: u16) -> Option<usize> {
        Some(4usize)
            .and_then(|size| size.checked_mul(width as usize))
            .and_then(|size| size.checked_mul(height as usize))
    }

    #[inline(always)]
    fn pixel_indices(&self, x: u16, y: u16) -> Option<Range<usize>> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(self.pixel_indices_unchecked(x, y))
    }

    #[inline(always)]
    fn pixel_indices_unchecked(&self, x: u16, y: u16) -> Range<usize> {
        let no_channels = 4;
        // If in bounds, this can't overflow as we have tested that at construction!
        let min_index = (y as usize * self.width as usize + x as usize) * no_channels;
        min_index..min_index + no_channels
    }

    /// # Panics
    ///
    /// Panics when the resulting image is larger than the maximum size of a vector.
    pub fn from_pixel(width: u16, height: u16, pixel: [u8; 4]) -> Self {
        let mut buf = Self::new(width, height);
        for p in buf.pixels_mut() {
            *p = pixel;
        }
        buf
    }

    /// The arguments to the function are the pixel's x and y coordinates.
    ///
    /// # Panics
    ///
    /// Panics when the resulting image is larger than the maximum size of a vector.
    pub fn from_fn<F>(width: u16, height: u16, mut f: F) -> Self
    where
        F: FnMut(u16, u16) -> [u8; 4],
    {
        let mut buf = Self::new(width, height);
        for (x, y, p) in buf.enumerate_pixels_mut() {
            *p = f(x, y);
        }
        buf
    }
}

/// GIF decoder
pub struct GifDecoder<R: GifRead> {
    reader: crate::Decoder<R>,
}

impl<R: GifRead> GifDecoder<R> {
    /// Creates a new decoder that decodes the input steam `r`
    pub fn new(r: R) -> Result<GifDecoder<R>, DecodingError> {
        let mut decoder = crate::DecodeOptions::new();
        decoder.set_color_output(ColorOutput::RGBA);

        Ok(GifDecoder {
            reader: decoder.read_info(r)?,
        })
    }

    pub fn dimensions(&self) -> (u16, u16) {
        (self.reader.width(), self.reader.height())
    }
}

pub struct GifFrameIterator<R: GifRead> {
    reader: crate::Decoder<R>,

    width: u16,
    height: u16,

    non_disposed_frame: Option<RgbaImageBuffer>,
    // `is_end` is used to indicate whether the iterator has reached the end of the frames.
    is_end: bool,
}

impl<R: GifRead> GifFrameIterator<R> {
    fn new(decoder: GifDecoder<R>) -> GifFrameIterator<R> {
        let (width, height) = decoder.dimensions();

        // intentionally ignore the background color for web compatibility

        GifFrameIterator {
            reader: decoder.reader,
            width,
            height,
            non_disposed_frame: None,
            is_end: false,
        }
    }
}

impl<R: GifRead> Iterator for GifFrameIterator<R> {
    type Item = Result<RgbaFrame, DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end {
            return None;
        }

        // Allocate the buffer for the previous frame.
        // This is done here and not in the constructor because
        // the constructor cannot return an error when the allocation limit is exceeded.
        if self.non_disposed_frame.is_none() {
            self.non_disposed_frame = Some(RgbaImageBuffer::from_pixel(
                self.width,
                self.height,
                [0, 0, 0, 0],
            ));
        }
        let non_disposed_frame = self.non_disposed_frame.as_mut().unwrap();

        let frame = match self.reader.next_frame_info() {
            Ok(frame_info) => {
                if let Some(frame) = frame_info {
                    FrameInfo::new_from_frame(frame)
                } else {
                    // no more frames
                    return None;
                }
            }
            Err(err) => {
                if matches!(err, DecodingError::UnexpectedEof) {
                    // end of file reached, no more frames
                    self.is_end = true;
                }
                return Some(Err(err));
            }
        };

        // Allocate the buffer now that the limits allowed it
        let mut vec = vec![0; self.reader.buffer_size()];
        if let Err(err) = self.reader.read_into_buffer(&mut vec) {
            return Some(Err(err));
        }

        // create the image buffer from the raw frame.
        // `buffer_size` uses wrapping arithmetic, thus might not report the
        // correct storage requirement if the result does not fit in `usize`.
        // on the other hand, `ImageBuffer::from_raw` detects overflow and
        // reports by returning `None`.
        let Some(mut frame_buffer) = RgbaImageBuffer::from_raw(frame.width, frame.height, vec)
        else {
            return Some(Err(DecodingError::format("Image dimensions are too large")));
        };

        // blend the current frame with the non-disposed frame, then update
        // the non-disposed frame according to the disposal method.
        fn blend_and_dispose_pixel(
            dispose: DisposalMethod,
            previous: &mut [u8; 4],
            current: &mut [u8; 4],
        ) {
            let pixel_alpha = current[3];
            if pixel_alpha == 0 {
                *current = *previous;
            }

            match dispose {
                DisposalMethod::Any | DisposalMethod::Keep => {
                    // do not dispose
                    // (keep pixels from this frame)
                    // note: the `Any` disposal method is underspecified in the GIF
                    // spec, but most viewers treat it identically to `Keep`
                    *previous = *current;
                }
                DisposalMethod::Background => {
                    // restore to background color
                    // (background shows through transparent pixels in the next frame)
                    *previous = [0, 0, 0, 0];
                }
                DisposalMethod::Previous => {
                    // restore to previous
                    // (dispose frames leaving the last none disposal frame)
                }
            }
        }

        // if `frame_buffer`'s frame exactly matches the entire image, then
        // use it directly, else create a new buffer to hold the composited
        // image.
        let image_buffer = if (frame.left, frame.top) == (0, 0)
            && (self.width, self.height) == (frame_buffer.width, frame_buffer.height)
        {
            for (x, y, pixel) in frame_buffer.enumerate_pixels_mut() {
                let previous_pixel = non_disposed_frame.get_pixel_mut(x, y);
                blend_and_dispose_pixel(frame.disposal_method, previous_pixel, pixel);
            }
            frame_buffer
        } else {
            RgbaImageBuffer::from_fn(self.width, self.height, |x, y| {
                let frame_x = x.wrapping_sub(frame.left);
                let frame_y = y.wrapping_sub(frame.top);
                let previous_pixel = non_disposed_frame.get_pixel_mut(x, y);

                if frame_x < frame_buffer.width && frame_y < frame_buffer.height {
                    let mut pixel = *frame_buffer.get_pixel(frame_x, frame_y);
                    blend_and_dispose_pixel(frame.disposal_method, previous_pixel, &mut pixel);
                    pixel
                } else {
                    // out of bounds, return pixel from previous frame
                    *previous_pixel
                }
            })
        };

        Some(Ok(RgbaFrame {
            buffer: image_buffer,
            delay: frame.delay,
        }))
    }
}

impl<R: GifRead> GifDecoder<R> {
    pub fn into_frames(self) -> GifFrameIterator<R> {
        GifFrameIterator::new(self)
    }
}

struct FrameInfo {
    left: u16,
    top: u16,
    width: u16,
    height: u16,
    disposal_method: DisposalMethod,
    delay: u64,
}

impl FrameInfo {
    fn new_from_frame(frame: &crate::common::GifFrame<'_>) -> FrameInfo {
        FrameInfo {
            left: frame.left,
            top: frame.top,
            width: frame.width,
            height: frame.height,
            disposal_method: frame.dispose,
            // frame.delay is in units of 10ms so frame.delay*10 is in ms
            delay: u64::from(frame.delay) * 10,
        }
    }
}
