//! Vetores, retângulos e utilidades numéricas.
//!
//! `Scalar` é um alias deliberado: o MVP usa `f32`, mas a simulação está
//! preparada para migrar para ponto fixo se o multiplayer passar a exigir
//! lockstep bit-exato.

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

pub type Scalar = f32;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    pub const fn splat(value: Scalar) -> Self {
        Self { x: value, y: value }
    }

    pub fn length_squared(self) -> Scalar {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> Scalar {
        self.length_squared().sqrt()
    }

    pub fn normalized_or_zero(self) -> Self {
        let length = self.length();
        if length > 0.0 {
            self / length
        } else {
            Self::ZERO
        }
    }

    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<Scalar> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Scalar) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<Scalar> for Vec2 {
    type Output = Self;

    fn div(self, rhs: Scalar) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    pub x: Scalar,
    pub y: Scalar,
    pub w: Scalar,
    pub h: Scalar,
}

impl Rect {
    pub const fn new(x: Scalar, y: Scalar, w: Scalar, h: Scalar) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_center(center: Vec2, size: Vec2) -> Self {
        Self::new(
            center.x - size.x * 0.5,
            center.y - size.y * 0.5,
            size.x,
            size.y,
        )
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    pub fn left(self) -> Scalar {
        self.x
    }

    pub fn right(self) -> Scalar {
        self.x + self.w
    }

    pub fn top(self) -> Scalar {
        self.y
    }

    pub fn bottom(self) -> Scalar {
        self.y + self.h
    }

    /// Sobreposição estrita: encostar na borda não conta.
    pub fn overlaps(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    pub fn expanded(self, amount: Scalar) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.w + amount * 2.0,
            self.h + amount * 2.0,
        )
    }
}

pub fn clamp(value: Scalar, min: Scalar, max: Scalar) -> Scalar {
    value.max(min).min(max)
}

pub fn approach(current: Scalar, target: Scalar, delta: Scalar) -> Scalar {
    let delta = delta.abs();
    if current < target {
        (current + delta).min(target)
    } else {
        (current - delta).max(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_arithmetic() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.length(), 5.0);
        assert_eq!(a + Vec2::new(1.0, 1.0), Vec2::new(4.0, 5.0));
        assert_eq!(a * 2.0, Vec2::new(6.0, 8.0));
        assert_eq!(Vec2::ZERO.normalized_or_zero(), Vec2::ZERO);
    }

    #[test]
    fn rect_overlap_is_strict() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 0.0, 10.0, 10.0);
        let c = Rect::new(9.0, 9.0, 2.0, 2.0);
        assert!(!a.overlaps(b));
        assert!(a.overlaps(c));
        assert!(a.contains(Vec2::new(0.0, 0.0)));
        assert!(!a.contains(Vec2::new(11.0, 0.0)));
    }

    #[test]
    fn approach_respects_target() {
        assert_eq!(approach(0.0, 10.0, 3.0), 3.0);
        assert_eq!(approach(9.0, 10.0, 3.0), 10.0);
        assert_eq!(approach(10.0, 0.0, 3.0), 7.0);
        assert_eq!(approach(1.0, 0.0, 3.0), 0.0);
    }
}
