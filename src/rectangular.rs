use std::ops::{Add, AddAssign};

use mvbitfield::prelude::*;
use strum::EnumIter;

use crate::{Rotation, Side};

pub enum RectangularSide {
    Vertical(VerticalSide),
    Horizontal(HorizontalSide),
}

bitfield! {
    pub enum VerticalSide: 1 {
        Right,
        Left,
    }

    pub enum HorizontalSide: 1 {
        Top,
        Bottom,
    }
}

impl VerticalSide {
    pub fn rotation_from_right(self) -> RectangularRotation {
        self.to_bitint().into()
    }

    pub fn to_square(self) -> Side {
        match self {
            Self::Right => Side::Right,
            Self::Left => Side::Left,
        }
    }
}

impl Add<RectangularRotation> for VerticalSide {
    type Output = Self;

    fn add(self, rhs: RectangularRotation) -> Self {
        self.to_bitint().wrapping_add(rhs.to_bitint()).into()
    }
}

impl AddAssign<RectangularRotation> for VerticalSide {
    fn add_assign(&mut self, rhs: RectangularRotation) {
        *self = *self + rhs;
    }
}

impl HorizontalSide {
    pub fn rotation_from_top(self) -> RectangularRotation {
        self.to_bitint().into()
    }

    pub fn to_square(self) -> Side {
        match self {
            Self::Top => Side::Top,
            Self::Bottom => Side::Bottom,
        }
    }
}

impl Add<RectangularRotation> for HorizontalSide {
    type Output = Self;

    fn add(self, rhs: RectangularRotation) -> Self {
        self.to_bitint().wrapping_add(rhs.to_bitint()).into()
    }
}

impl AddAssign<RectangularRotation> for HorizontalSide {
    fn add_assign(&mut self, rhs: RectangularRotation) {
        *self = *self + rhs;
    }
}

pub trait SideExt {
    fn to_rectangular(self) -> RectangularSide;
}

impl SideExt for Side {
    fn to_rectangular(self) -> RectangularSide {
        match self {
            Side::Right => RectangularSide::Vertical(VerticalSide::Right),
            Side::Top => RectangularSide::Horizontal(HorizontalSide::Top),
            Side::Left => RectangularSide::Vertical(VerticalSide::Left),
            Side::Bottom => RectangularSide::Horizontal(HorizontalSide::Bottom),
        }
    }
}

bitfield! {
    #[derive(PartialOrd, Ord, EnumIter)]
    pub enum RectangularRotation: 1 {
        Identity,
        HalfTurn,
    }
}

impl RectangularRotation {
    pub fn to_square(self) -> Rotation {
        Rotation::new_masked(2 * self.to_primitive())
    }
}

impl Add for RectangularRotation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.to_bitint().wrapping_add(rhs.to_bitint()).into()
    }
}

impl AddAssign for RectangularRotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod tests {
    use crate::{Rotation, Side};

    use super::{HorizontalSide, RectangularRotation, VerticalSide};

    #[test]
    fn rotation_from_right() {
        assert_eq!(
            VerticalSide::Right.rotation_from_right(),
            RectangularRotation::Identity,
        );
        assert_eq!(
            VerticalSide::Left.rotation_from_right(),
            RectangularRotation::HalfTurn,
        );
    }

    #[test]
    fn rotation_from_top() {
        assert_eq!(
            HorizontalSide::Top.rotation_from_top(),
            RectangularRotation::Identity,
        );
        assert_eq!(
            HorizontalSide::Bottom.rotation_from_top(),
            RectangularRotation::HalfTurn,
        );
    }

    #[test]
    fn to_square() {
        assert_eq!(VerticalSide::Right.to_square(), Side::Right);
        assert_eq!(HorizontalSide::Top.to_square(), Side::Top);
        assert_eq!(VerticalSide::Left.to_square(), Side::Left);
        assert_eq!(HorizontalSide::Bottom.to_square(), Side::Bottom);
        assert_eq!(
            RectangularRotation::Identity.to_square(),
            Rotation::Identity,
        );
        assert_eq!(
            RectangularRotation::HalfTurn.to_square(),
            Rotation::HalfTurn,
        );
    }

    #[test]
    fn add() {
        assert_eq!(
            VerticalSide::Right + RectangularRotation::Identity,
            VerticalSide::Right,
        );
        assert_eq!(
            VerticalSide::Right + RectangularRotation::HalfTurn,
            VerticalSide::Left,
        );
        assert_eq!(
            VerticalSide::Left + RectangularRotation::Identity,
            VerticalSide::Left,
        );
        assert_eq!(
            VerticalSide::Left + RectangularRotation::HalfTurn,
            VerticalSide::Right,
        );
        assert_eq!(
            HorizontalSide::Top + RectangularRotation::Identity,
            HorizontalSide::Top,
        );
        assert_eq!(
            HorizontalSide::Top + RectangularRotation::HalfTurn,
            HorizontalSide::Bottom,
        );
        assert_eq!(
            HorizontalSide::Bottom + RectangularRotation::Identity,
            HorizontalSide::Bottom,
        );
        assert_eq!(
            HorizontalSide::Bottom + RectangularRotation::HalfTurn,
            HorizontalSide::Top,
        );
    }

    #[test]
    fn add_assign() {
        let mut side = VerticalSide::Right;
        side += RectangularRotation::Identity;
        assert_eq!(side, VerticalSide::Right);
        side += RectangularRotation::HalfTurn;
        assert_eq!(side, VerticalSide::Left);
        side += RectangularRotation::Identity;
        assert_eq!(side, VerticalSide::Left);
        side += RectangularRotation::HalfTurn;
        assert_eq!(side, VerticalSide::Right);

        let mut side = HorizontalSide::Top;
        side += RectangularRotation::Identity;
        assert_eq!(side, HorizontalSide::Top);
        side += RectangularRotation::HalfTurn;
        assert_eq!(side, HorizontalSide::Bottom);
        side += RectangularRotation::Identity;
        assert_eq!(side, HorizontalSide::Bottom);
        side += RectangularRotation::HalfTurn;
        assert_eq!(side, HorizontalSide::Top);

        let mut rotation = RectangularRotation::Identity;
        rotation += RectangularRotation::Identity;
        assert_eq!(rotation, RectangularRotation::Identity);
        rotation += RectangularRotation::HalfTurn;
        assert_eq!(rotation, RectangularRotation::HalfTurn);
        rotation += RectangularRotation::Identity;
        assert_eq!(rotation, RectangularRotation::HalfTurn);
        rotation += RectangularRotation::HalfTurn;
        assert_eq!(rotation, RectangularRotation::Identity);
    }
}
