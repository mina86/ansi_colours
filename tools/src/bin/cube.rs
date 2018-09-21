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

static CUBE_VALUES: [u8; 6] = [0, 95, 135, 175, 215, 255];

fn distance(x: &lab::Lab, y: (u8, u8, u8)) -> f32 {
    delta_e::DE2000::new(*x, lab::Lab::from_rgb(&[y.0, y.1, y.2]))
}

fn find_best<F: Fn(u8) -> (u8, u8, u8)>(component: &'static str, to_rgb: F) {
    print!("{:5}: ", component);
    let colours: std::vec::Vec<_> = CUBE_VALUES.iter().map(|v| {
        let (r, g, b) = to_rgb(*v);
        lab::Lab::from_rgb(&[r, g, b])
    }).collect();
    let mut last = 0;
    for want in 0..256 {
        let (i, _) = colours.iter().enumerate().map(|(i, colour)| (
            i, distance(colour, to_rgb(want as u8))
        )).min_by(|x, y| if x.1 < y.1 {
            std::cmp::Ordering::Less
        } else if x.1 > y.1 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }).unwrap();
        if i != last {
            print!("{}{:3}", if last == 0 { "" } else {", " }, want);
            last = i;
        }
    }
    print!("\n");
}

fn main() {
    find_best("red", |r| (r, 0, 0));
    find_best("green", |g| (0, g, 0));
    find_best("blue", |b| (0, 0, b));
}
