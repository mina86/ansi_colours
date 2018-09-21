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

extern crate delta_e;
extern crate lab;


fn luminance_mean(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + b as u16) / 3) as u8
}

fn luminance_mean2g(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + 2 * g as u16 + b as u16) / 4) as u8
}

fn luminance_weight(r: u8, g: u8, b: u8) -> u8 {
    let y = 3568058 * r as u32 + 11998262 * g as u32 + 1210896 * b as u32;
    (y >> 24) as u8
    // FYI, the above gives the same results as:
    //     let y = 2647777 * r as u32 + 8903644 * g as u32 + 898579 * b as u32;
    //     (y / 12450000) as u8
    // and:
    //     let y = 7943331 * r as u64 + 26710933 * g as u64 + 2695736 * b as u64
    //     (y / 37350000) as u8
}

fn luminance_square(r: u8, g: u8, b: u8) -> u8 {
    ((r as f32 * r as f32 * 0.21267285140562248 +
      g as f32 * g as f32 * 0.715152155287818   +
      b as f32 * b as f32 * 0.07217499330655958).sqrt() + 0.5) as u8
}

fn luminance_isqrt(r: u8, g: u8, b: u8) -> u8 {
    fn isqrt(n: u16) -> u8 {
        let mut c: u16 = 0x80;
        let mut g: u16 = 0;

        while c != 0 {
            g |= c;
            if g * g > n {
                g ^= c;
            }
            c >>= 1;
        }
        g as u8
    }

    isqrt(((r as u32 * r as u32 * 13938 +
            g as u32 * g as u32 * 46868 +
            b as u32 * b as u32 *  4730) >> 16) as u16)
}

fn luminance_xyz(r: u8, g: u8, b: u8) -> u8 {
    from_linear(0.21267285140562248 * to_linear(r) +
                0.715152155287818   * to_linear(g) +
                0.07217499330655958 * to_linear(b))
}

fn to_linear(v: u8) -> f32 {
    if v <= 10 {
        v as f32 * (1.0 / 3294.6)
    } else {
        ((v as f32 + 14.025) * (1.0 / 269.025)).powf(2.4)
    }
}

fn from_linear(v: f32) -> u8 {
    (if v <= 0.0031308 {
        3294.6 * v
    } else {
        v.powf(1.0 / 2.4) * 269.025 - 14.025
    } + 0.5) as u8
}

fn distance(r: u8, g: u8, b: u8, grey: u8) -> f32 {
    fn lab_from_y(y: f32) -> lab::Lab {
        // http://www.brucelindbloom.com/index.html?Eqn_XYZ_to_Lab.html
        let l = if y > 216.0 / 24389.0 {
            116.0 * y.powf(1.0 / 3.0) - 16.0
        } else {
            24389.0 / 27.0 * y
        };
        lab::Lab { l: l, a: 0.0, b: 0.0 }
    }

    let y = 0.212672851 * to_linear(r) +
        0.715152155 * to_linear(g) +
        0.072174993 * to_linear(b);
    delta_e::DE2000::new(lab_from_y(y), lab_from_y(to_linear(grey)))
}

fn try_function(name: &str, f: &Fn(u8, u8, u8)->u8) {
    let start = std::time::Instant::now();
    let mut hash: u32 = 0;
    for c in 0..(1 << 24) {
        hash = hash.overflowing_mul(17).0.overflowing_add(
            f((c >> 16) as u8, (c >> 8) as u8, c as u8) as u32).0;
    }
    let elapsed = start.elapsed();
    let elapsed = elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64;

    let mut total_dist = 0.0;
    let mut max_dist = 0.0;
    let mut n = 0;
    for c in 0..(1 << 24) {
        let (r, g, b) = ((c >> 16) as u8, (c >> 8) as u8, c as u8);
        let d = distance(r, g, b, f(r, g, b)) as f64;
        if d > max_dist {
            max_dist = d;
        }
        total_dist += d;
        n += 1;
    }

    print!("{} {:5} ms;  distance: avg={:12.9} max={:12.9};  [{:x}]",
           name, elapsed, total_dist / n as f64, max_dist, hash);

    let black = f(0, 0, 0);
    let white = f(255, 255, 255);
    if black != 0 {
        print!(" black={}", black);
    }
    if white != 255 {
        print!(" white={}", white);
    }
    println!();
}

fn main() {
    try_function("mean  ", &luminance_mean);
    try_function("mean2g", &luminance_mean2g);
    try_function("weight", &luminance_weight);
    try_function("square", &luminance_square);
    try_function("isqrt ", &luminance_isqrt );
    try_function("xyz   ", &luminance_xyz);
}
