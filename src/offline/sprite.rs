use image::DynamicImage;

/// In Vec<Option<OfflineSprite>>, outer Option indicates state of the fetch -
/// whether the network request has happened yet, and whether it was successful.
/// the inner Option indicates whether or not the pokemon variant itself HAS a sprite.
#[derive(Debug, Clone)]
pub struct OfflineSprite {
    pub sprite: Option<DynamicImage>,
}
