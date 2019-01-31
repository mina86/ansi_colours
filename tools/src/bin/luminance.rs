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


fn luminance_average(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + b as u16) / 3) as u8
}

fn luminance_average_2g(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + 2 * g as u16 + b as u16) / 4) as u8
}

fn luminance_average_16(r: u8, g: u8, b: u8) -> u8 {
    let y = 54 * r as u16 + 183 * g as u16 + 19 * b as u16;
    (y >> 8) as u8
}

fn luminance_average_32(r: u8, g: u8, b: u8) -> u8 {
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

fn luminance_gamma22(r: u8, g: u8, b: u8) -> u8 {
    fn to_linear(v: u8) -> f32 {
        (v as f32 / 255.0).powf(2.2)
    }

    fn from_linear(v: f32) -> u8 {
        ((v.powf(1.0 / 2.2) * 255.0) + 0.5) as u8
    }

    from_linear(0.21267285140562248 * to_linear(r) +
                0.715152155287818   * to_linear(g) +
                0.07217499330655958 * to_linear(b))
}

fn luminance_xyz(r: u8, g: u8, b: u8) -> u8 {
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

    from_linear(0.21267285140562248 * to_linear(r) +
                0.715152155287818   * to_linear(g) +
                0.07217499330655958 * to_linear(b))
}

fn distance(r: u8, g: u8, b: u8, grey: u8) -> f32 {
    fn to_linear(v: u8) -> f32 {
        if v <= 10 {
            v as f32 * (1.0 / 3294.6)
        } else {
            ((v as f32 + 14.025) * (1.0 / 269.025)).powf(2.4)
        }
    }

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

struct Histogram {
    name: &'static str,
    data: [u32; 101],
}

fn measure_function(name: &'static str, f: &Fn(u8, u8, u8)->u8) -> Histogram {
    // Measure time
    let start = std::time::Instant::now();
    let mut hash: u32 = 0;
    for c in 0..(1 << 24) {
        hash = hash.overflowing_mul(17).0.overflowing_add(
            f((c >> 16) as u8, (c >> 8) as u8, c as u8) as u32).0;
    }
    let elapsed = start.elapsed();
    let elapsed = elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64;

    // Calculate statistics
    let mut histogram = [0; 101];
    let mut total_dist = 0.0;
    let mut max_dist = 0.0;
    for c in 0..(1 << 24) {
        let (r, g, b) = ((c >> 16) as u8, (c >> 8) as u8, c as u8);
        let d = distance(r, g, b, f(r, g, b)) as f64;
        histogram[std::cmp::min(d as usize, histogram.len() - 1)] += 1;
        if d > max_dist {
            max_dist = d;
        }
        total_dist += d;
    }

    print!("{} {:5} ms;  distance: avg={:9.6} max={:9.6};  [{:x}]",
           name, elapsed, total_dist / (1 << 24) as f64, max_dist, hash);

    let black = f(0, 0, 0);
    let white = f(255, 255, 255);
    if black != 0 {
        print!(" black={}", black);
    }
    if white != 255 {
        print!(" white={}", white);
    }
    println!();

    Histogram { name: name, data: histogram }
}

fn main() {
    let mut histograms = Vec::new();

    println!("Benchmarks");
    histograms.push(measure_function("average     ", &luminance_average));
    histograms.push(measure_function("… w/ 2*green", &luminance_average_2g));
    histograms.push(measure_function("… 16-bit alu", &luminance_average_16));
    histograms.push(measure_function("… 32-bit alu", &luminance_average_32));
    histograms.push(measure_function("γ=2.0       ", &luminance_square));
    histograms.push(measure_function("γ=2 (no fpu)", &luminance_isqrt));
    histograms.push(measure_function("γ=2.2       ", &luminance_gamma22));
    histograms.push(measure_function("precise     ", &luminance_xyz));

    println!("\nHistogram        d<1   1≤d<2   2≤d<3   3≤d<4   4≤d<5   5≤d<6   6≤d<7   7≤d");
    for histogram in histograms.iter() {
        print!("{}" , histogram.name);
        let mut n = 8;
        while histogram.data[n - 1] == 0 {
            n -= 1;
        }
        let mut left = 1 << 24;
        for count in histogram.data.iter().take(n - 1) {
            print!(" {:6.2}%", *count as f64 * 100.0 / (1 << 24) as f64);
            left -= count;
        }
        print!(" {:6.2}%", left as f64 * 100.0 / (1 << 24) as f64);
        println!();
    }

    println!("\nHistogram CSV");
    for histogram in histograms.iter() {
        print!("\"{}\"" , histogram.name);
        for count in histogram.data.iter() {
            print!(",{}", *count);
        }
        println!();
    }
}
