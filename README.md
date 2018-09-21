# True-colour ↔ ANSI terminal palette converter

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

This library has C, C++ and Rust bindings and can be easily used from
any of those languages.

### Rust

Using this package with Cargo projects is as simple as adding a single
dependency:

```toml
[dependencies]
ansi_colours = "^1.0"
```

and then using one of the two functions that the library provides:

```rust
extern crate ansi_colours;

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

### C and C++

The easiest way to use this package in C or C++ is to copy the
`ansi_colour.h` and `ansi256.c` files to your project (unfortunately,
C nor C++ has any centralised package repository), set up compilation
step for the `ansi256.c` file, add header file to include path and
once all that is done use the two functions provided by this library
can be used:

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

C nor C++ ecosystem has a centralised library distribution service
which is why at this stage the easiest way is to just copy the two
required files.
