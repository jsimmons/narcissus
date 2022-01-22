#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Quat {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat44([f32; 16]);

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Mat43([f32; 12]);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
