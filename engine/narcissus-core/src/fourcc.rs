#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct FourCC(u32);

impl FourCC {
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> u32 {
        self.0
    }
}

impl std::fmt::Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_le_bytes();
        if let Ok(str) = std::str::from_utf8(&bytes) {
            write!(f, "FourCC('{str}')",)
        } else {
            write!(f, "FourCC(0x{:08X})", self.0)
        }
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(value: [u8; 4]) -> Self {
        FourCC::from_raw(u32::from_le_bytes(value))
    }
}

#[macro_export]
macro_rules! fourcc {
    ($code:literal) => {
        if $code.as_bytes().len() == 4 {
            FourCC::from_raw(u32::from_le_bytes([
                $code.as_bytes()[0],
                $code.as_bytes()[1],
                $code.as_bytes()[2],
                $code.as_bytes()[3],
            ]))
        } else {
            panic!("invalid fourcc code")
        }
    };
}

#[cfg(test)]
mod tests {
    use super::FourCC;

    #[test]
    fn basic() {
        let magic = fourcc!("DDS ");
        const MAGIC: FourCC = fourcc!("DDS ");
        assert_eq!(magic.0, 0x20534444);
        assert_eq!(MAGIC.0, 0x20534444);
    }
}
