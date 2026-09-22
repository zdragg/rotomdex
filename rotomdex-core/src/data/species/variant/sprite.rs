use core::task::{Context, Poll};

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use embassy_time::Duration;
use relative_path::RelativePath;
use rotomdex_api::Client;
use rotomdex_api::model::pokemon::PokemonSprites;
use snafu::{ResultExt, Snafu};
use tracing::{Instrument, Span};

use crate::Settings;
use crate::data::resource::{ArbitraryResourceError, Fetchable, ResourceResult};

#[derive(Debug, Clone)]
pub(crate) struct ModelSprite {
    image: Option<RgbaImage>,
    animated: Option<Animation>,
}

#[derive(Debug, Snafu)]
enum SpriteError {
    #[snafu(display("cannot decode png: {source}"))]
    Minipng { source: minipng::Error },
    #[snafu(display("cannot decode gif: {source}"))]
    Gif { source: rotomdex_gif::DecodingError },
}

impl ArbitraryResourceError for SpriteError {}

impl Fetchable for ModelSprite {
    type Request = PokemonSprites;
    async fn fetch(
        request: Self::Request,
        client: Client,
        _settings: Settings,
    ) -> ResourceResult<Self> {
        let image: ResourceResult<Option<RgbaImage>> = async {
            if let Some(image_link) = request.front_default {
                let png_bytes = client.get_bytes(RelativePath::new(&image_link)).await?;

                let png_header = minipng::decode_png_header(&png_bytes).context(MinipngSnafu)?;
                let mut buffer = vec![0; png_header.required_bytes_rgba8bpc()];

                let mut image =
                    minipng::decode_png(&png_bytes, &mut buffer).context(MinipngSnafu)?;
                image.convert_to_rgba8bpc().context(MinipngSnafu)?;

                let image = RgbaImage::from(image);

                Ok(Some(if let Some(bounds) = image.get_bounds() {
                    image.crop(bounds)
                } else {
                    image
                }))
            } else {
                tracing::warn!("no sprite found");
                Ok(None)
            }
        }
        .instrument(tracing::info_span!("fetch_static"))
        .await;

        let animation: ResourceResult<Option<Animation>> = async {
            if let Some(animation_link) = request.other.showdown.front_default {
                let animation_bytes = client.get_bytes(RelativePath::new(&animation_link)).await?;

                let decoder =
                    rotomdex_gif::GifDecoder::new(animation_bytes.as_ref()).context(GifSnafu)?;

                let frames_result: Result<Vec<_>, rotomdex_gif::DecodingError> =
                    decoder.into_frames().collect();
                let frames = frames_result.context(GifSnafu)?;

                Ok(Some(frames.into()))
            } else {
                tracing::warn!("no sprite found");
                Ok(None)
            }
        }
        .instrument(tracing::info_span!("fetch_animated"))
        .await;

        Ok(ModelSprite {
            image: image?,
            animated: animation?,
        })
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn fetch_span(_request: &Self::Request) -> Span {
        tracing::info_span!("fetch_sprite")
    }
}

impl ModelSprite {
    pub(crate) fn image(&self) -> Option<&RgbaImage> {
        self.image.as_ref()
    }

    pub(crate) fn animated(&self) -> Option<&Animation> {
        self.animated.as_ref()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RgbaImage {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl From<rotomdex_gif::RgbaImageBuffer> for RgbaImage {
    fn from(buffer: rotomdex_gif::RgbaImageBuffer) -> Self {
        Self {
            width: buffer.width as usize,
            height: buffer.height as usize,
            pixels: buffer.data,
        }
    }
}

impl From<minipng::ImageData<'_>> for RgbaImage {
    fn from(image: minipng::ImageData) -> Self {
        Self {
            width: image.width() as usize,
            height: image.height() as usize,
            pixels: image.pixels().to_owned(),
        }
    }
}

impl RgbaImage {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
        }
    }

    pub(crate) fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn as_raw(&self) -> &[u8] {
        &self.pixels
    }
}

#[derive(Clone, Copy)]
struct ImageBounds {
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

impl ImageBounds {
    fn union(&self, other: &Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

impl RgbaImage {
    /// Returns `None` if fully transparent.
    fn get_bounds(&self) -> Option<ImageBounds> {
        let mut bounds: Option<ImageBounds> = None;
        for (index, pixel) in self.pixels.as_chunks::<4>().0.iter().enumerate() {
            if pixel[3] == 0 {
                continue;
            }

            let x = index % self.width;
            let y = index / self.width;
            let pixel_bounds = ImageBounds {
                min_x: x,
                min_y: y,
                max_x: x,
                max_y: y,
            };
            bounds = Some(match bounds {
                Some(bounds) => bounds.union(&pixel_bounds),
                None => pixel_bounds,
            });
        }
        bounds
    }

    fn crop(&self, bounds: ImageBounds) -> Self {
        let ImageBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        } = bounds;
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let side = width.max(height) + 2;
        let left = min_x as i64 - (side - width) as i64 / 2;
        let top = min_y as i64 - (side - height) as i64 / 2;

        let mut cropped = Self::new(side, side);
        let start_x = left.max(0);
        let end_x = (left + side as i64).min(self.width as i64);
        let start_y = top.max(0);
        let end_y = (top + side as i64).min(self.height as i64);

        if start_x >= end_x {
            return cropped;
        }

        let row_bytes = (end_x - start_x) as usize * 4;
        for y in start_y..end_y {
            let source = (y as usize * self.width + start_x as usize) * 4;
            let target = ((y - top) as usize * side + (start_x - left) as usize) * 4;
            cropped.pixels[target..target + row_bytes]
                .copy_from_slice(&self.pixels[source..source + row_bytes]);
        }
        cropped
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Animation {
    frames: Vec<AnimationFrame>,
    duration: Duration,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimationFrame {
    image: RgbaImage,
    ends_at: Duration,
}

impl Animation {
    pub(crate) fn frame_at(&self, duration: Duration) -> &RgbaImage {
        if self.duration == Duration::MIN {
            return &self.frames.first().unwrap().image;
        }
        let duration = duration.as_ticks() % self.duration.as_ticks();
        &self
            .frames
            .iter()
            .find(|frame| frame.ends_at.as_ticks() > duration)
            .unwrap()
            .image
    }
}

impl From<Vec<rotomdex_gif::RgbaFrame>> for Animation {
    fn from(frames: Vec<rotomdex_gif::RgbaFrame>) -> Self {
        let frames: Vec<_> = frames
            .into_iter()
            .map(|frame| (frame.delay, RgbaImage::from(frame.buffer)))
            .collect();

        let bounds = frames
            .iter()
            .filter_map(|(_, image)| image.get_bounds())
            .reduce(|bounds, next| bounds.union(&next));

        let mut total_duration = Duration::default();
        let frames: Vec<_> = frames
            .into_iter()
            .map(|(delay, image)| {
                total_duration += Duration::from_millis(delay);
                let image = if let Some(bounds) = bounds {
                    image.crop(bounds)
                } else {
                    image
                };
                AnimationFrame {
                    ends_at: total_duration,
                    image,
                }
            })
            .collect();

        Self {
            frames,
            duration: total_duration,
        }
    }
}
