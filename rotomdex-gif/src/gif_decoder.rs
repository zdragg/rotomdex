//! Decoding of GIF Images

use alloc::vec::Vec;
use embedded_io::BufRead;

use crate::reader::DecoderIter;
use crate::{ColorOutput, DecodeOptions, Decoder, DecodingError, DisposalMethod, Frame};

/// GIF decoder
pub struct GifDecoder<R: BufRead<Error: Send + Sync + 'static>> {
    reader: Decoder<R>,
}

impl<R: BufRead<Error: Send + Sync + 'static>> GifDecoder<R> {
    /// Creates a new decoder that decodes the input steam `r`
    pub fn new(r: R) -> Result<Self, DecodingError> {
        let mut decoder = DecodeOptions::new();
        decoder.set_color_output(ColorOutput::RGBA);

        Ok(GifDecoder {
            reader: decoder.read_info(r)?,
        })
    }

    pub fn into_frames(self) -> GifFrameIterator<R> {
        GifFrameIterator::new(self)
    }
}

pub struct GifFrameIterator<R: BufRead<Error: Send + Sync + 'static>> {
    reader: DecoderIter<R>,
    width: u16,
    height: u16,
    non_disposed_frame: Option<Vec<u8>>,
    // `is_end` is used to indicate whether the iterator has reached the end of the frames.
    // Or encounter any un-recoverable error.
    is_end: bool,
}

impl<R: BufRead<Error: Send + Sync + 'static>> GifFrameIterator<R> {
    fn new(decoder: GifDecoder<R>) -> Self {
        // intentionally ignore the background color for web compatibility
        GifFrameIterator {
            width: decoder.reader.width(),
            height: decoder.reader.height(),
            reader: decoder.reader.into_iter(),
            non_disposed_frame: None,
            is_end: false,
        }
    }
}

fn canvas_buffer(width: u16, height: u16) -> Result<Vec<u8>, DecodingError> {
    let size = usize::from(width)
        .checked_mul(usize::from(height))
        .and_then(|size| size.checked_mul(4))
        .ok_or(DecodingError::MemoryLimit)?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(size)
        .map_err(|_| DecodingError::OutOfMemory)?;
    buffer.resize(size, 0);
    Ok(buffer)
}

impl<R: BufRead<Error: Send + Sync + 'static>> Iterator for GifFrameIterator<R> {
    type Item = Result<Frame<'static>, DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end {
            return None;
        }

        let frame = match self.reader.next()? {
            Ok(frame) => frame,
            Err(err) => return Some(Err(err)),
        };
        let result = self.compose(frame);
        self.is_end = result.is_err();
        Some(result)
    }
}

impl<R: BufRead<Error: Send + Sync + 'static>> GifFrameIterator<R> {
    fn compose(&mut self, frame: Frame<'static>) -> Result<Frame<'static>, DecodingError> {
        if self.non_disposed_frame.is_none() {
            self.non_disposed_frame = Some(canvas_buffer(self.width, self.height)?);
        }
        // Bind to a variable to avoid repeated `.unwrap()` calls
        let non_disposed_frame = self.non_disposed_frame.as_mut().unwrap();
        let mut frame_buffer = frame.buffer.into_owned();

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
            && (self.width, self.height) == (frame.width, frame.height)
        {
            for (previous_pixel, pixel) in non_disposed_frame
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(frame_buffer.as_chunks_mut::<4>().0.iter_mut())
            {
                blend_and_dispose_pixel(frame.dispose, previous_pixel, pixel);
            }
            frame_buffer
        } else {
            let mut image_buffer = canvas_buffer(self.width, self.height)?;
            let width = usize::from(self.width);
            let frame_width = usize::from(frame.width);
            let frame_height = usize::from(frame.height);
            let left = usize::from(frame.left);
            let top = usize::from(frame.top);
            let previous_pixels = non_disposed_frame.as_chunks_mut::<4>().0;
            let frame_pixels = frame_buffer.as_chunks::<4>().0;
            for (index, pixel) in image_buffer.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let x = index % width;
                let y = index / width;
                let frame_x = x.wrapping_sub(left);
                let frame_y = y.wrapping_sub(top);
                let previous_pixel = &mut previous_pixels[index];

                if frame_x < frame_width && frame_y < frame_height {
                    *pixel = frame_pixels[frame_y * frame_width + frame_x];
                    blend_and_dispose_pixel(frame.dispose, previous_pixel, pixel);
                } else {
                    // out of bounds, return pixel from previous frame
                    *pixel = *previous_pixel;
                }
            }
            image_buffer
        };

        Ok(Frame {
            width: self.width,
            height: self.height,
            delay: frame.delay,
            buffer: image_buffer.into(),
            ..Frame::default()
        })
    }
}
