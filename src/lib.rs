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

//! `ansi_colours` converts between 24-bit sRGB colours and 8-bit colour palette
//! used by ANSI terminals such as xterm on rxvt-unicode in 256-colour mode.
//! The most common use case is when using 24-bit colours in a terminal emulator
//! which only support 8-bit colour palette.  It allows true-colours to be
//! approximated by values supported by the terminal.
//!
//! When mapping true-colour into available 256-colour palette, it tries to
//! balance accuracy and performance.  It doesn’t implement the fastest
//! algorithm nor is it the most accurate, instead it uses a formula which
//! should be fast enough and accurate enough for most use-cases.
//!
//! ## Cargo features
//!
//! To facilitate better interoperability the crate defines `rgb` crate
//! feature (enabled by default).  It adds support for the `RGB` type from
//! [`rgb` crate](https://crates.io/crates/rgb).  Specifically, `RGB8`
//! (a.k.a. `RGB<u8>`) as well as `RGB16` (a.k.a. `RGB<u16>`) types are
//! supported.
//!
//! Furthermore, `ansi_term` and `termcolor` features are available.  They
//! add support for `Colour` type from [`ansi_term`
//! crate](https://crates.io/crates/ansi_term) and `Color` type from
//! [`termcolor` crate](https://crates.io/crates/termcolor) respectively.
//! This includes support for calling `ansi256_from_rgb` with arguments of
//! those types and implementation of `ColourExt` trait which extends the
//! types with additional conversion methods.
//!
//! ## Usage
//!
//! Using this library with Cargo projects is as simple as adding a single
//! dependency:
//!
//! ```toml
//! [dependencies]
//! ansi_colours = "1.1"
//! ```
//!
//! and then using one of the two functions that the library provides:
//!
//! ```rust
//! use ansi_colours::*;
//!
//! fn main() {
//!     // Colour at given index:
//!     println!("{:-3}: {:?}", 50, ansi_colours::rgb_from_ansi256(50));
//!
//!     // Approximate true-colour by colour in the palette:
//!     let rgb = (100, 200, 150);
//!     let index = ansi256_from_rgb(rgb);
//!     println!("{:?} ~ {:-3} {:?}",
//!              rgb, index, ansi_colours::rgb_from_ansi256(index));
//! }
//! ```

#![no_std]

mod ansi256;
mod impls;
#[cfg(test)]
mod test;

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
    let rgb = ansi256::ANSI_COLOURS[idx as usize];
    ((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Returns index of a colour in 256-colour ANSI palette approximating given
/// sRGB colour.
///
/// Because the first 16 colours of the palette are not standardised and usually
/// user-configurable, the function usually ignores them.
///
/// Th first argument uses [`AsRGB`] trait so that the function can be called in
/// multiple ways using different representations of RGB colours such as
/// `0xRRGGBB` integer, `(r, g, b)` tuple or `[r, g, b]` array.  Calling the
/// function is equivalent to calling [`AsRGB::to_ansi256`] method.
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
pub fn ansi256_from_rgb<C: AsRGB>(rgb: C) -> u8 { rgb.to_ansi256() }

/// Returns index of a colour in 256-colour ANSI palette approximating given
/// shade of grey.
///
/// A shade of grey is an sRGB colour whose red, green and blue components are
/// the same.  The function takes that component value as an argument.  It gives
/// the same results as `ansi256_from_rgb((c, c, c))` but is faster.
///
/// # Examples
///
///
/// ```
/// assert_eq!( 16, ansi_colours::ansi256_from_grey(0));
/// assert_eq!( 16, ansi_colours::ansi256_from_grey(1));
/// assert_eq!(231, ansi_colours::ansi256_from_grey(255));
/// ```
#[inline]
pub fn ansi256_from_grey(component: u8) -> u8 {
    ansi256::ANSI256_FROM_GREY[component as usize]
}

/// Type which (can) represent an sRGB colour.  Used to provide overloaded
/// versions of `ansi256_from_rgb` function.
pub trait AsRGB {
    /// Returns representation of the sRGB colour as a 24-bit `0xRRGGBB`
    /// integer.
    fn as_u32(&self) -> u32;

    /// Returns index of a colour in 256-colour ANSI palette approximating given
    /// sRGB colour.
    ///
    /// This is provided by default and uses [`Self::as_u32`] to determine
    /// 24-bit sRGB representation of the colour.  The method should be
    /// implemented for types which have multiple representations one of which
    /// stores index in the palette which can be returned directly.
    #[inline]
    fn to_ansi256(&self) -> u8 {
        crate::ansi256::ansi256_from_rgb(self.as_u32())
    }
}

/// Extension to types representing ANSI colours adding methods converting
/// between RGB and indexed (a.k.a. fixed) representations.
pub trait ColourExt: Sized {
    /// Constructs an indexed colour which approximates given sRGB colour.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ansi_colours::ColourExt;
    /// use ansi_term::Colour;
    ///
    /// assert_eq!(Colour::Fixed( 16), Colour::approx_rgb(  0,   0,   0));
    /// assert_eq!(Colour::Fixed( 16), Colour::approx_rgb(  0,   1,   2));
    /// assert_eq!(Colour::Fixed( 67), Colour::approx_rgb( 95, 135, 175));
    /// assert_eq!(Colour::Fixed(231), Colour::approx_rgb(255, 255, 255));
    /// ```
    ///
    /// Note that the example requires `ansi_term` cargo feature to be enabled.
    fn approx_rgb(r: u8, g: u8, b: u8) -> Self;

    /// Constructs an indexed colour which approximates given sRGB colour.
    ///
    /// Behaves like [`approx_rgb`](`Self::approx_rgb`) but takes a single
    /// argument which implements [`AsRGB`].  Note that types which implement
    /// `ColourExt` typically also implement `AsRGB` which means this method can
    /// be called with `Self` argument.  It’s usually better to call
    /// [`to_256`](`ColourExt::to_256`) instead.
    #[inline]
    fn approx<C: AsRGB>(rgb: C) -> Self {
        let rgb = rgb.as_u32();
        Self::approx_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

    /// Converts the colour into 256-colour-compatible format.
    ///
    /// If the colour represents an RGB colour, converts it into indexed
    /// representation using [`ansi256_from_rgb`] function.  Otherwise, returns
    /// the colour unchanged.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ansi_colours::ColourExt;
    /// use ansi_term::Colour;
    ///
    /// assert_eq!(Colour::Red,        Colour::Red.to_256());
    /// assert_eq!(Colour::Fixed( 11), Colour::Fixed(11).to_256());
    /// assert_eq!(Colour::Fixed( 16), Colour::RGB(  0,   0,   0).to_256());
    /// assert_eq!(Colour::Fixed( 16), Colour::RGB(  0,   1,   2).to_256());
    /// assert_eq!(Colour::Fixed( 67), Colour::RGB( 95, 135, 175).to_256());
    /// assert_eq!(Colour::Fixed(231), Colour::RGB(255, 255, 255).to_256());
    /// ```
    ///
    /// Note that the example requires `ansi_term` cargo feature to be enabled.
    fn to_256(&self) -> Self;

    /// Converts the colour colour into sRGB.
    ///
    /// Named colours (black, red etc. through white) are treated like indexed
    /// colours with indexes 0 through 7.  Indexed colours are converted into
    /// sRGB using [`rgb_from_ansi256`] function.  RGB colours are returned
    /// unchanged.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ansi_colours::ColourExt;
    /// use ansi_term::Colour;
    ///
    /// assert_eq!((  0,   0,   0), Colour::Fixed( 16).to_rgb());
    /// assert_eq!(( 95, 135, 175), Colour::Fixed( 67).to_rgb());
    /// assert_eq!((255, 255, 255), Colour::Fixed(231).to_rgb());
    /// assert_eq!((238, 238, 238), Colour::Fixed(255).to_rgb());
    /// assert_eq!(( 42,  24,   0), Colour::RGB(42, 24, 0).to_rgb());
    /// ```
    ///
    /// Note that the example requires `ansi_term` cargo feature to be enabled.
    fn to_rgb(&self) -> (u8, u8, u8);
}
