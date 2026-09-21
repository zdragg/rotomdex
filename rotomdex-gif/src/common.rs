use alloc::borrow::Cow;
use alloc::vec::Vec;

/// Convenience trait used to mark all readers accepted by decoders.
pub trait GifRead: embedded_io::BufRead<Error: Sync + Send + 'static> {}
impl<R: embedded_io::BufRead<Error: Sync + Send + 'static>> GifRead for R {}

/// Disposal method
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DisposalMethod {
    /// `StreamingDecoder` is not required to take any action.
    Any = 0,
    /// Do not dispose.
    Keep = 1,
    /// Restore to background color.
    Background = 2,
    /// Restore to previous.
    Previous = 3,
}

impl DisposalMethod {
    /// Converts `u8` to `Option<Self>`
    #[must_use]
    pub const fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Any),
            1 => Some(Self::Keep),
            2 => Some(Self::Background),
            3 => Some(Self::Previous),
            _ => None,
        }
    }
}

/// Known GIF block labels.
///
/// Note that the block uniquely specifies the layout of bytes that follow and how they are
/// framed. For example, the header always has a fixed length but is followed by a variable amount
/// of additional data. An image descriptor may be followed by a local color table depending on
/// information read in it. Therefore, it doesn't make sense to continue parsing after encountering
/// an unknown block as the semantics of following bytes are unclear.
///
/// The extension block provides a common framing for an arbitrary amount of application specific
/// data which may be ignored.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    /// Image block.
    Image = 0x2C,
    /// Extension block.
    Extension = 0x21,
    /// Image trailer.
    Trailer = 0x3B,
}

impl Block {
    /// Converts `u8` to `Option<Self>`
    #[must_use]
    pub const fn from_u8(n: u8) -> Option<Self> {
        match n {
            0x2C => Some(Self::Image),
            0x21 => Some(Self::Extension),
            0x3B => Some(Self::Trailer),
            _ => None,
        }
    }
}

/// A newtype wrapper around an arbitrary extension ID.
///
/// An extension is some amount of byte data organized in sub-blocks so that one can skip over it
/// without knowing the semantics. Though technically you likely want to use a `Application`
/// extension, the library tries to stay flexible here.
///
/// This allows us to customize the set of impls compared to a raw `u8`. It also clarifies the
/// intent and gives some inherent methods for interoperability with known extension types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnyExtension(pub u8);

/// Known GIF extension labels.
///
/// These are extensions which may be interpreted by the library and to which a specification with
/// the internal data layout is known.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Extension {
    /// Plain Text extension.
    ///
    /// This instructs the decoder to render a text as characters in a grid of cells, in a
    /// mono-spaced font of its choosing. This is seldom actually implemented and ignored by
    /// ImageMagick. The color is always taken from the global table which further complicates any
    /// use. No real information on the frame sequencing of this block is available in the
    /// standard.
    Text = 0x01,
    /// Control extension.
    Control = 0xF9,
    /// Comment extension.
    Comment = 0xFE,
    /// Application extension.
    ///
    /// See [ImageMagick] for an idea of commonly recognized extensions.
    ///
    /// [ImageMagick]: https://github.com/ImageMagick/ImageMagick/blob/b0b58c6303195928060f55f9c3ca8233ab7f7733/coders/gif.c#L1128
    Application = 0xFF,
}

impl AnyExtension {
    /// Decode the label as a known extension.
    #[must_use]
    pub const fn into_known(self) -> Option<Extension> {
        Extension::from_u8(self.0)
    }
}

impl From<Extension> for AnyExtension {
    fn from(ext: Extension) -> Self {
        Self(ext as u8)
    }
}

impl Extension {
    /// Converts `u8` to a `Extension` if it is known.
    #[must_use]
    pub const fn from_u8(n: u8) -> Option<Self> {
        match n {
            0x01 => Some(Self::Text),
            0xF9 => Some(Self::Control),
            0xFE => Some(Self::Comment),
            0xFF => Some(Self::Application),
            _ => None,
        }
    }
}

/// A GIF frame
#[derive(Debug, Clone)]
pub struct GifFrame<'a> {
    /// Frame delay in units of 10 ms.
    pub delay: u16,
    /// Disposal method.
    pub dispose: DisposalMethod,
    /// Transparent index (if available).
    pub transparent: Option<u8>,
    /// True if the frame needs user input to be displayed.
    pub needs_user_input: bool,
    /// Offset from the top border of the canvas.
    pub top: u16,
    /// Offset from the left border of the canvas.
    pub left: u16,
    /// Width of the frame.
    pub width: u16,
    /// Height of the frame.
    pub height: u16,
    /// True if the image is interlaced.
    pub interlaced: bool,
    /// Frame local color palette if available.
    pub palette: Option<Vec<u8>>,
    /// Buffer containing the image data.
    /// Only indices unless configured differently.
    pub buffer: Cow<'a, [u8]>,
}

impl Default for GifFrame<'_> {
    fn default() -> Self {
        GifFrame {
            delay: 0,
            dispose: DisposalMethod::Keep,
            transparent: None,
            needs_user_input: false,
            top: 0,
            left: 0,
            width: 0,
            height: 0,
            interlaced: false,
            palette: None,
            buffer: Cow::Borrowed(&[]),
        }
    }
}
impl GifFrame<'static> {
    /// Leaves empty buffer and empty palette behind
    #[inline]
    pub(crate) fn take(&mut self) -> Self {
        GifFrame {
            delay: self.delay,
            dispose: self.dispose,
            transparent: self.transparent,
            needs_user_input: self.needs_user_input,
            top: self.top,
            left: self.left,
            width: self.width,
            height: self.height,
            interlaced: self.interlaced,
            palette: core::mem::take(&mut self.palette),
            buffer: core::mem::replace(&mut self.buffer, Cow::Borrowed(&[])),
        }
    }
}

/// Number of repetitions
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Repeat {
    /// Finite number of repetitions
    Finite(u16),
    /// Infinite number of repetitions
    Infinite,
}

impl Default for Repeat {
    fn default() -> Self {
        Self::Finite(0)
    }
}
