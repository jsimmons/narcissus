#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Quat {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}
