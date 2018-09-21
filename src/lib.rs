// ansi_colours – true-colour ↔ ANSI terminal palette converter
// Copyright 2018 by Michał Nazarewicz <mina86@mina86.com>
//
// ansi_colours is free software: you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 3 of the License, or (at
// your option) any later version.
//
// ansi_colours is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser
// General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with ansi_colours.  If not, see <http://www.gnu.org/licenses/>.

//! `ansi_colours` is a library which converts between 24-bit sRGB colours and
//! 8-bit colour palette used by ANSI terminals such as xterm on rxvt-unicode in
//! 256-colour mode.
//!
//! The most common use case is when using 24-bit colours in a terminal emulator
//! which only support 8-bit colour palette.  This package allows true-colours
//! to be approximated by values supported by the terminal.
//!
//! When mapping true-colour into available 256-colour palette (of which only
//! 240 are actually usable), this package tries to balance accuracy and
//! performance.  It doesn’t implement the fastest algorithm nor is it the most
//! accurate, instead it uses a formula which should be fast enough and accurate
//! enough for most use-cases.
//!
//! ## Usage
//!
//! Using this library with Cargo projects is as simple as adding a single
//! dependency:
//!
//! ```toml
//! [dependencies]
//! ansi_colours = "^1.0"
//! ```
//!
//! and then using one of the two functions that the library provides:
//!
//! ```rust
//! extern crate ansi_colours;
//!
//! use ansi_colours::*;
//!
//! fn main() {
//!     // Colour at given index:
//!     println!("{:-3}: {:?}", 50, rgb_from_ansi256(50));
//!
//!     // Approximate true-colour by colour in the palette:
//!     let rgb = (100, 200, 150);
//!     let index = ansi256_from_rgb(rgb);
//!     println!("{:?} ~ {:-3} {:?}", rgb, index, rgb_from_ansi256(index));
//! }
//! ```

mod externs;

/// Returns sRGB colour corresponding to the index in the 256-colour ANSI
/// palette.
///
/// The first 16 colours (so-called system colours) are not standardised and
/// terminal emulators often allow them to be customised.  Because of this,
/// their value should not be relied upon.  For system colours, this function
/// returns default colours used by XTerm.
///
/// Remaining 240 colours consist of a 6×6×6 colour cube and a 24-step greyscale
/// ramp.  Those are standardised and thus should be the same on every terminal
/// which supports 256-colour colour palette.
///
/// # Examples
///
///
/// ```
/// assert_eq!((  0,   0,   0), ansi_colours::rgb_from_ansi256( 16));
/// assert_eq!(( 95, 135, 175), ansi_colours::rgb_from_ansi256( 67));
/// assert_eq!((255, 255, 255), ansi_colours::rgb_from_ansi256(231));
/// assert_eq!((238, 238, 238), ansi_colours::rgb_from_ansi256(255));
/// ```
#[inline]
pub fn rgb_from_ansi256(idx: u8) -> (u8, u8, u8) {
    let rgb = unsafe { externs::rgb_from_ansi256(idx) };
    ((rgb >> 16) as u8, (rgb >>  8) as u8, rgb as u8)
}

/// Returns index of a colour in 256-colour ANSI palette approximating given
/// sRGB colour.
///
/// Because the first 16 colours of the palette are not standardised and usually
/// user-configurable, the function essentially ignores them.
///
/// Th first argument uses `AsRGB` trait so that the function can be called in
/// multiple ways using different representations of RGB colours such as
/// `0xRRGGBB` integer, `(r, g, b)` tuple or `[r, g, b]` array.
///
/// # Examples
///
///
/// ```
/// assert_eq!( 16, ansi_colours::ansi256_from_rgb(0x000000));
/// assert_eq!( 16, ansi_colours::ansi256_from_rgb( (  1,   1,   1)));
/// assert_eq!( 16, ansi_colours::ansi256_from_rgb( [  0,   1,   2]));
/// assert_eq!( 67, ansi_colours::ansi256_from_rgb(&( 95, 135, 175)));
/// assert_eq!(231, ansi_colours::ansi256_from_rgb(&[255, 255, 255]));
/// ```
#[inline]
pub fn ansi256_from_rgb<C: AsRGB>(rgb: C) -> u8 {
    unsafe { externs::ansi256_from_rgb(rgb.as_u32()) }
}

/// A trait for types which (can) represent an sRGB colour.  Used to provide
/// overloaded versions of `ansi256_from_rgb` function.
pub trait AsRGB {
    /// Returns representation of the sRGB colour as a 24-bit `0xRRGGBB`
    /// integer.
    fn as_u32(&self) -> u32;
}

/// Representation of an RGB colour as 24-bit `0xRRGGBB` integer.
impl AsRGB for u32 {
    fn as_u32(&self) -> u32 { *self }
}

#[inline]
fn to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

impl AsRGB for (u8, u8, u8) {
    fn as_u32(&self) -> u32 { to_u32(self.0, self.1, self.2) }
}

impl AsRGB for [u8; 3] {
    fn as_u32(&self) -> u32 { to_u32(self[0], self[1], self[2]) }
}

impl<'a, T: AsRGB + ?Sized> AsRGB for &'a T {
    fn as_u32(&self) -> u32 { (*self).as_u32() }
}
