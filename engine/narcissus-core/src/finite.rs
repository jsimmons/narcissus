use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct NotFiniteError;

impl Display for NotFiniteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for NotFiniteError {}

/// A floating point value that is gauranteed to be finite.
///
/// This allows us to safely implement Hash, Eq and Ord.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FiniteF32(f32);

impl FiniteF32 {
    #[inline(always)]
    pub fn new(x: f32) -> Result<FiniteF32, NotFiniteError> {
        if x.is_finite() {
            Ok(FiniteF32(x))
        } else {
            Err(NotFiniteError)
        }
    }

    #[inline(always)]
    pub fn get(self) -> f32 {
        self.0
    }
}

impl PartialEq for FiniteF32 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FiniteF32 {}

impl PartialOrd for FiniteF32 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FiniteF32 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SAFETY: There are no NaNs since FiniteF32 is always finite.
        unsafe { self.0.partial_cmp(&other.0).unwrap_unchecked() }
    }
}

impl std::hash::Hash for FiniteF32 {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // `Hash` requires that if `a == b` then `hash(a) == hash(b)`. In IEEE-754
        // floating point `0.0 == -0.0`, so we must normalize the value before hashing.
        let x = if self.0 == 0.0 { 0.0 } else { self.0 };
        x.to_bits().hash(state);
    }
}

/// A floating point value that is gauranteed to be finite.
///
/// This allows us to safely implement Hash, Eq and Ord.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FiniteF64(f64);

impl FiniteF64 {
    #[inline(always)]
    pub fn new(x: f64) -> Result<FiniteF64, NotFiniteError> {
        if x.is_finite() {
            Ok(FiniteF64(x))
        } else {
            Err(NotFiniteError)
        }
    }

    #[inline(always)]
    pub fn get(self) -> f64 {
        self.0
    }
}

impl PartialEq for FiniteF64 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FiniteF64 {}

impl PartialOrd for FiniteF64 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FiniteF64 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SAFETY: There are no NaNs since FiniteF32 is always finite.
        unsafe { self.0.partial_cmp(&other.0).unwrap_unchecked() }
    }
}

impl std::hash::Hash for FiniteF64 {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash requires that if `a == b` then `hash(a) == hash(b)`.
        // In ieee 754 floating point `0.0 == -0.0`, so we must normalize the value before hashing.
        let x = if self.0 == 0.0 { 0.0 } else { self.0 };
        x.to_bits().hash(state);
    }
}
