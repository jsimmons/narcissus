use narcissus_font::{Font, FontCollection};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum FontFamily {
    NotoSansJapanese,
    RobotoRegular,
}

pub struct Fonts<'a> {
    noto_sans_japanese: Font<'a>,
    roboto_regular: Font<'a>,
}

impl<'a> Fonts<'a> {
    pub fn new() -> Self {
        // SAFETY: Safe because Roboto-Regular.ttf is a valid ttf font embedded
        // in the application.
        let roboto_regular =
            unsafe { Font::from_bytes(include_bytes!("fonts/Roboto-Regular.ttf")) };
        let noto_sans_japanese =
            unsafe { Font::from_bytes(include_bytes!("fonts/NotoSansJP-Medium.otf")) };
        Self {
            noto_sans_japanese,
            roboto_regular,
        }
    }
}

impl<'a> FontCollection<'a> for Fonts<'a> {
    type Family = FontFamily;
    fn font(&self, family: Self::Family) -> &Font<'a> {
        match family {
            FontFamily::NotoSansJapanese => &self.noto_sans_japanese,
            FontFamily::RobotoRegular => &self.roboto_regular,
        }
    }
}
