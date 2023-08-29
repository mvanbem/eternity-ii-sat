use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::Color;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayEdge<const N: usize>([Color; N]);

impl<const N: usize> ArrayEdge<N> {
    pub fn new(colors: [Color; N]) -> Self {
        Self(colors)
    }

    /// # Panics
    ///
    /// Panics if any byte in `colors` is not a valid [`Color`].
    pub fn from_byte_string(colors: &[u8; N]) -> Self {
        let mut result = Self::default();
        for (index, b) in colors.iter().copied().enumerate() {
            result[index] = Color::from_byte_char(b).unwrap();
        }
        result
    }

    pub fn reversed(&self) -> Self {
        let mut result = Self::default();
        for (index, c) in self.0.into_iter().rev().enumerate() {
            result[index] = c;
        }
        result
    }

    pub fn flip_eq(&self, rhs: &Self) -> bool {
        self.0.iter().zip(rhs.0.iter().rev()).all(|(a, b)| a == b)
    }

    pub fn iter(&self) -> impl Iterator<Item = Color> + '_ {
        self.0.iter().copied()
    }
}

impl<const N: usize> Default for ArrayEdge<N> {
    fn default() -> Self {
        Self([Color::EXTERIOR; N])
    }
}

impl<const N: usize> Index<usize> for ArrayEdge<N> {
    type Output = Color;

    fn index(&self, index: usize) -> &Color {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for ArrayEdge<N> {
    fn index_mut(&mut self, index: usize) -> &mut Color {
        &mut self.0[index]
    }
}

impl<const N: usize> Debug for ArrayEdge<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(&self, f)
    }
}

impl<const N: usize> Display for ArrayEdge<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for color in self.0.iter() {
            write!(f, "{}", color.to_char())?;
        }
        Ok(())
    }
}
