use crate::gamma;
use core::ops::{Div, Index, IndexMut, Mul};
use micromath::F32Ext as _;

#[repr(transparent)]
pub struct Image([Color; 64]);

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    pub fn gamma_correct(&self) -> Self {
        Color {
            r: gamma::gamma_correct(self.r),
            g: gamma::gamma_correct(self.g),
            b: gamma::gamma_correct(self.b),
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: (self.r as f32 * rhs).clamp(0.0, 255.0).round() as u8,
            g: (self.g as f32 * rhs).clamp(0.0, 255.0).round() as u8,
            b: (self.b as f32 * rhs).clamp(0.0, 255.0).round() as u8,
        }
    }
}

impl Div<f32> for Color {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Color {
            r: (self.r as f32 / rhs).clamp(0.0, 255.0).round() as u8,
            g: (self.g as f32 / rhs).clamp(0.0, 255.0).round() as u8,
            b: (self.b as f32 / rhs).clamp(0.0, 255.0).round() as u8,
        }
    }
}

impl Image {
    pub const fn new_solid(color: Color) -> Self {
        Image([color; 64])
    }

    pub fn row(&self, row: usize) -> &[Color] {
        &self.0[(8 * row)..(8 * (row + 1))]
    }

    pub fn gradient(color: Color) -> Self {
        let mut image = Image::new_solid(color);
        for row in 0..8 {
            for col in 0..8 {
                image[(row, col)] = color / ((1 + row * row + col) as f32);
            }
        }
        image
    }
}

impl Default for Image {
    fn default() -> Self {
        Image::new_solid(Color::default())
    }
}

impl Index<(usize, usize)> for Image {
    type Output = Color;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (row, col) = index;
        &self.0[row * 8 + col]
    }
}

impl IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (row, col) = index;
        &mut self.0[row * 8 + col]
    }
}

impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe { core::mem::transmute(self) }
    }
}

impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe { core::mem::transmute(self) }
    }
}
