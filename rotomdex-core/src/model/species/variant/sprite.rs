use std::task::{Context, Poll};

use image::{DynamicImage, GenericImageView, Pixel};
use rustemon::model::pokemon::PokemonSprites;

use crate::FetchContext;
use crate::model::Fetchable;

#[derive(Debug, Clone)]
pub(crate) struct ModelSprite {
    pub(crate) sprite: Option<DynamicImage>,
}

impl Fetchable for ModelSprite {
    type Request = PokemonSprites;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> color_eyre::eyre::Result<Self> {
        let Some(link) = request.front_default else {
            // log::warn!("sprite not found: {}", request.); TODO: somehow get name here
            return Ok(Self { sprite: None });
        };

        let crop = |image: DynamicImage| -> DynamicImage {
            let (mut min_x, mut max_x, mut min_y, mut max_y) = (image.width(), 0, image.height(), 0);
            for (x, y, color) in image.pixels() {
                if color.alpha() == 0 {
                    continue;
                }
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
            let (mid_x, mid_y) = ((min_x + max_x) / 2, (min_y + max_y) / 2);
            let side_len = (max_x - min_x).max(max_y - min_y) + 2; // leave some space
            let (corner_x, corner_y) = (mid_x.saturating_sub(side_len / 2), mid_y.saturating_sub(side_len / 2));
            image.crop_imm(corner_x, corner_y, side_len, side_len)
        };
        let image_bytes = ctx.req_client.get(link).send().await?.bytes().await?;
        let image = crop(image::load_from_memory(&image_bytes)?);

        Ok(ModelSprite { sprite: Some(image) })
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn is_loaded(&self) -> bool {
        true
    }
}
