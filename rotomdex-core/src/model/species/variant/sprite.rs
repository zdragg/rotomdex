use std::io::Cursor;
use std::task::{Context, Poll};
use std::time::Duration;

use color_eyre::eyre::Result;
use image::codecs::gif;
use image::{AnimationDecoder, Frame, RgbaImage, imageops};
use rustemon::model::pokemon::PokemonSprites;

use crate::FetchContext;
use crate::model::Fetchable;

#[derive(Debug, Clone)]
pub(crate) struct ModelSprite {
    image: Option<RgbaImage>,
    animated: Option<ModelGif>,
}

impl Fetchable for ModelSprite {
    type Request = PokemonSprites;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let image = {
            if let Some(image_link) = request.front_default {
                let image_bytes = ctx.req_client.get(image_link).send().await?.bytes().await?;
                let image = image::load_from_memory(&image_bytes)?.into_rgba8();
                Some(if let Some(bounds) = bbox(&image) {
                    crop(&image, bounds)
                } else {
                    image
                })
            } else {
                // log::warn!("sprite not found: {}", request.); TODO: somehow get name here
                None
            }
        };
        let animated = if let Some(animated_link) = request.other.showdown.front_default {
            let bytes = ctx.req_client.get(animated_link).send().await?.bytes().await?;

            let frames = gif::GifDecoder::new(Cursor::new(bytes))?
                .into_frames()
                .collect_frames()?;
            Some(frames.into())
        } else {
            None
        };

        Ok(ModelSprite { image, animated })
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn is_loaded(&self) -> bool {
        true
    }
}
impl ModelSprite {
    pub(crate) fn image(&self) -> Option<&RgbaImage> {
        self.image.as_ref()
    }

    pub(crate) fn animated(&self) -> Option<&ModelGif> {
        self.animated.as_ref()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ModelGif {
    frames: Vec<ModelGifFrame>,
    duration: Duration,
}

#[derive(Debug, Clone)]
pub(crate) struct ModelGifFrame {
    image: RgbaImage,
    ends_at: Duration,
}

impl ModelGif {
    pub(crate) fn frame_at(&self, duration: Duration) -> &RgbaImage {
        let duration = duration.as_nanos() % self.duration.as_nanos();
        &self
            .frames
            .iter()
            .find(|frame| frame.ends_at.as_nanos() > duration)
            .unwrap()
            .image
    }
}

impl From<Vec<Frame>> for ModelGif {
    fn from(value: Vec<Frame>) -> Self {
        let bounds = value.iter().filter_map(|frame| bbox(frame.buffer())).reduce(
            |(min_x, min_y, max_x, max_y), (next_min_x, next_min_y, next_max_x, next_max_y)| {
                (
                    min_x.min(next_min_x),
                    min_y.min(next_min_y),
                    max_x.max(next_max_x),
                    max_y.max(next_max_y),
                )
            },
        );

        let mut total_duration = Duration::default();
        let frames: Vec<_> = value
            .into_iter()
            .map(|frame| {
                total_duration += Duration::from(frame.delay());
                let image = if let Some(bounds) = bounds {
                    crop(frame.buffer(), bounds)
                } else {
                    frame.into_buffer()
                };
                ModelGifFrame {
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

fn bbox(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] == 0 {
            continue;
        }

        bounds = Some(match bounds {
            Some((min_x, min_y, max_x, max_y)) => (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y)),
            None => (x, y, x, y),
        });
    }

    bounds
}

fn crop(image: &RgbaImage, (min_x, min_y, max_x, max_y): (u32, u32, u32, u32)) -> RgbaImage {
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let side = width.max(height) + 2;
    let left = min_x as i64 - (side - width) as i64 / 2;
    let top = min_y as i64 - (side - height) as i64 / 2;

    let mut cropped = RgbaImage::new(side, side);
    imageops::replace(&mut cropped, image, -left, -top);
    cropped
}
