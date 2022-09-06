#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Quat {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Quat {
    pub const ZERO: Self = Self {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    };

    pub const IDENTITY: Self = Self {
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
    };

    pub const NAN: Self = Self {
        a: std::f32::NAN,
        b: std::f32::NAN,
        c: std::f32::NAN,
        d: std::f32::NAN,
    };
}
