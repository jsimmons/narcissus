use crate::{HalfTurn, Vec3, sin_cos_pi_f32};

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
        a: f32::NAN,
        b: f32::NAN,
        c: f32::NAN,
        d: f32::NAN,
    };

    /// Create a new quaternion from its components.
    #[inline(always)]
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { a, b, c, d }
    }

    /// Returns a quaternion representing a `rotation` in half turns around the
    /// given `axis`.
    pub fn from_axis_rotation(axis: Vec3, rotation: HalfTurn) -> Self {
        let (s, c) = sin_cos_pi_f32(rotation.as_f32() * 0.5);
        let v = axis * s;
        Self {
            a: v.x,
            b: v.y,
            c: v.z,
            d: c,
        }
    }

    /// Rotates `rhs` by `self`.
    pub fn transform_vec3(self, rhs: Vec3) -> Vec3 {
        let d = self.d;
        let v = Vec3::new(self.a, self.b, self.c);
        rhs * (d * d - Vec3::dot(v, v))
            + v * Vec3::dot(rhs, v) * 2.0
            + Vec3::cross(v, rhs) * d * 2.0
    }
}

impl std::ops::Mul<Vec3> for Quat {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self.transform_vec3(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::{HalfTurn, Quat, Vec3};

    #[test]
    fn constructors() {
        assert_eq!(
            Quat::from_axis_rotation(Vec3::X, HalfTurn(1.0)),
            Quat::new(1.0, 0.0, 0.0, 0.0)
        );
        assert_eq!(
            Quat::from_axis_rotation(Vec3::Y, HalfTurn(1.0)),
            Quat::new(0.0, 1.0, 0.0, 0.0)
        );
        assert_eq!(
            Quat::from_axis_rotation(Vec3::Z, HalfTurn(1.0)),
            Quat::new(0.0, 0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn multiplication() {
        let rot_180_x = Quat::from_axis_rotation(Vec3::X, HalfTurn::new(1.0));
        assert_eq!(rot_180_x * Vec3::X, Vec3::X);
        assert_eq!(rot_180_x * Vec3::Y, -Vec3::Y);
        assert_eq!(rot_180_x * Vec3::Z, -Vec3::Z);
        let rot_180_y = Quat::from_axis_rotation(Vec3::Y, HalfTurn::new(1.0));
        assert_eq!(rot_180_y * Vec3::X, -Vec3::X);
        assert_eq!(rot_180_y * Vec3::Y, Vec3::Y);
        assert_eq!(rot_180_y * Vec3::Z, -Vec3::Z);
        let rot_180_z = Quat::from_axis_rotation(Vec3::Z, HalfTurn::new(1.0));
        assert_eq!(rot_180_z * Vec3::X, -Vec3::X);
        assert_eq!(rot_180_z * Vec3::Y, -Vec3::Y);
        assert_eq!(rot_180_z * Vec3::Z, Vec3::Z);
    }
}
