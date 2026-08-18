use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct OfflineSprite {
    pub sprite: Option<DynamicImage>,
}
