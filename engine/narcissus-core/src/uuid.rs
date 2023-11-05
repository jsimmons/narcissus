use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseUuidError;

impl Display for ParseUuidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided string is not a valid UUID".fmt(f)
    }
}

impl Error for ParseUuidError {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub fn nil() -> Self {
        Self([0; 16])
    }

    pub fn from_bytes_be(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn parse_str_unwrap(uuid: &str) -> Self {
        match Uuid::parse_str(uuid) {
            Ok(uuid) => uuid,
            Err(_) => panic!("provided string is not a valid UUID"),
        }
    }

    pub const fn parse_str(uuid: &str) -> Result<Self, ParseUuidError> {
        const LUT: [u8; 256] = [
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, !0, !0, !0, !0, !0, !0, !0, 10, 11, 12,
            13, 14, 15, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, 10, 11, 12, 13, 14, 15, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
            !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0,
        ];

        let len = uuid.len();
        let uuid = uuid.as_bytes();
        if len != 36 || uuid[8] != b'-' || uuid[13] != b'-' || uuid[18] != b'-' || uuid[23] != b'-'
        {
            return Err(ParseUuidError);
        }

        let h_00_0 = LUT[uuid[0] as usize];
        let h_00_1 = LUT[uuid[1] as usize];
        let h_01_0 = LUT[uuid[2] as usize];
        let h_01_1 = LUT[uuid[3] as usize];
        let h_02_0 = LUT[uuid[4] as usize];
        let h_02_1 = LUT[uuid[5] as usize];
        let h_03_0 = LUT[uuid[6] as usize];
        let h_03_1 = LUT[uuid[7] as usize];
        // -
        let h_04_0 = LUT[uuid[9] as usize];
        let h_04_1 = LUT[uuid[10] as usize];
        let h_05_0 = LUT[uuid[11] as usize];
        let h_05_1 = LUT[uuid[12] as usize];
        // -
        let h_06_0 = LUT[uuid[14] as usize];
        let h_06_1 = LUT[uuid[15] as usize];
        let h_07_0 = LUT[uuid[16] as usize];
        let h_07_1 = LUT[uuid[17] as usize];
        // -
        let h_08_0 = LUT[uuid[19] as usize];
        let h_08_1 = LUT[uuid[20] as usize];
        let h_09_0 = LUT[uuid[21] as usize];
        let h_09_1 = LUT[uuid[22] as usize];
        // -
        let h_10_0 = LUT[uuid[24] as usize];
        let h_10_1 = LUT[uuid[25] as usize];
        let h_11_0 = LUT[uuid[26] as usize];
        let h_11_1 = LUT[uuid[27] as usize];
        let h_12_0 = LUT[uuid[28] as usize];
        let h_12_1 = LUT[uuid[29] as usize];
        let h_13_0 = LUT[uuid[30] as usize];
        let h_13_1 = LUT[uuid[31] as usize];
        let h_14_0 = LUT[uuid[32] as usize];
        let h_14_1 = LUT[uuid[33] as usize];
        let h_15_0 = LUT[uuid[34] as usize];
        let h_15_1 = LUT[uuid[35] as usize];

        let bits = h_00_0
            | h_00_1
            | h_01_0
            | h_01_1
            | h_02_0
            | h_02_1
            | h_03_0
            | h_03_1
            | h_04_0
            | h_04_1
            | h_05_0
            | h_05_1
            | h_06_0
            | h_06_1
            | h_07_0
            | h_07_1
            | h_08_0
            | h_08_1
            | h_09_0
            | h_09_1
            | h_10_0
            | h_10_1
            | h_11_0
            | h_11_1
            | h_12_0
            | h_12_1
            | h_13_0
            | h_13_1
            | h_14_0
            | h_14_1
            | h_15_0
            | h_15_1;

        // Only possible if any of the half-words are invalid.
        if bits == !0 {
            return Err(ParseUuidError);
        }

        Ok(Self([
            h_00_0 << 4 | h_00_1,
            h_01_0 << 4 | h_01_1,
            h_02_0 << 4 | h_02_1,
            h_03_0 << 4 | h_03_1,
            // -
            h_04_0 << 4 | h_04_1,
            h_05_0 << 4 | h_05_1,
            // -
            h_06_0 << 4 | h_06_1,
            h_07_0 << 4 | h_07_1,
            // -
            h_08_0 << 4 | h_08_1,
            h_09_0 << 4 | h_09_1,
            // -
            h_10_0 << 4 | h_10_1,
            h_11_0 << 4 | h_11_1,
            h_12_0 << 4 | h_12_1,
            h_13_0 << 4 | h_13_1,
            h_14_0 << 4 | h_14_1,
            h_15_0 << 4 | h_15_1,
        ]))
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15],
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::uuid::ParseUuidError;

    use super::Uuid;

    #[test]
    fn test_uuid() {
        assert_eq!(
            Uuid::parse_str("00000000-0000-0000-0000-000000000000"),
            Ok(Uuid::nil())
        );
        assert_eq!(
            format!(
                "{}",
                Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
            ),
            "00000000-0000-0000-0000-000000000000"
        );

        assert_eq!(
            Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff"),
            Ok(Uuid::from_bytes_be([
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff
            ]))
        );
        assert_eq!(
            format!(
                "{}",
                Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").unwrap()
            ),
            "00112233-4455-6677-8899-aabbccddeeff"
        );

        assert_eq!(
            Uuid::parse_str("01234567-89AB-CDEF-0123-456789ABCDEF"),
            Ok(Uuid::from_bytes_be([
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
                0xcd, 0xef
            ]))
        );
        assert_eq!(
            format!(
                "{}",
                Uuid::parse_str("01234567-89AB-CDEF-0123-456789ABCDEF").unwrap()
            ),
            "01234567-89ab-cdef-0123-456789abcdef"
        );

        assert_eq!(Uuid::parse_str(""), Err(ParseUuidError));
        assert_eq!(
            Uuid::parse_str("ERROR000-0000-0000-0000-000000000000"),
            Err(ParseUuidError)
        );
    }
}
