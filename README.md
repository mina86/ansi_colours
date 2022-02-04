# True-colour ↔ ANSI terminal palette converter

[![crates.io](https://img.shields.io/crates/v/ansi_colours)](https://crates.io/crates/ansi_colours)
[![Docs](https://docs.rs/ansi_colours/badge.svg)](https://docs.rs/ansi_colours)
[![License](https://img.shields.io/badge/license-LGPL-blue.svg)](https://github.com/mina86/ansi_colours/blob/master/LICENSE)

`ansi_colours` is a library which converts between 24-bit sRGB colours
and 8-bit colour palette used by ANSI terminals such as xterm on
rxvt-unicode in 256-colour mode.

The most common use case is when using 24-bit colours in a terminal
emulator which only support 8-bit colour palette.  This package allows
true-colours to be approximated by values supported by the terminal.

When mapping true-colour into available 256-colour palette (of which
only 240 are actually usable), this package tries to balance accuracy
and performance.  It doesn’t implement the fastest algorithm nor is it
the most accurate, instead it uses a formula which should be fast
enough and accurate enough for most use-cases.

## Usage

This library has C and Rust implementations and can be easily used
from either of those languages (as well as from C++ of course).  The
two implementations are equivalent and are provided for best
performance.  Since version 1.0.4 the Rust crate has sped up by 25%
when doing True Colour → ANSI index conversion and 75% when doing
conversion in the other direction.

### Rust

Using this package with Cargo projects is as simple as adding a single
dependency:

```toml
[dependencies]
ansi_colours = "1.1"
```

and then using one of functions that the library provides:

```rust
use ansi_colours::*;

fn main() {
    // Colour at given index:
    println!("{:-3}: {:?}", 50, rgb_from_ansi256(50));

    // Approximate true-colour by colour in the palette:
    let rgb = (100, 200, 150);
    let index = ansi256_from_rgb(rgb);
    println!("{:?} ~ {:-3} {:?}", rgb, index, rgb_from_ansi256(index));
}
```

To facilitate better interoperability this library supports `rgb::RGB`
from [`rgb` crate](https://crates.io/crates/rgb).  The support is
controlled by `rgb` feature.  With it enabled (which is the default),
`ansi256_from_rgb` function accepts `RGB<u8>` and `RGB<u16>` values as
arguments (though `rgb_from_ansi256` still returns colours as tuples).

### C and C++

The easiest way to use this library in C or C++ is to copy the
`ansi_colour.h` and `ansi256.c` files to your project, set up
compilation step for the `ansi256.c` file, add the header file to the
include path and once all that is done use the two provided functions:

```c
#include <stdio.h>

#include "ansi_colours.h"

int main() {
    // Colour at given index:
    printf("%-3u #%06x\n",  50, rgb_from_ansi256(50));

    // Approximate true-colour by colour in the palette:
    uint32_t rgb = 0x64C896;
    uint8_t index = ansi256_from_rgb(rgb);
    printf("#%06x ~ %-3u %06x\n", rgb, index, rgb_from_ansi256(index));

    return 0;
}
```

Unfortunately neither C nor C++ ecosystem has a centralised package
distribution service so there currently is no more convenient
solution.
