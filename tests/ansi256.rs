fn to_rgb(index: u8) -> (u8, u8, u8) {
    ansi_colours::rgb_from_ansi256(index)
}

fn to_ansi(rgb: (u8, u8, u8)) -> u8 {
    ansi_colours::ansi256_from_rgb(rgb)
}

static CUBE_VALUES: [u8; 6] = [0, 95, 135, 175, 215, 255];

#[test]
fn test_to_rgb() {
    #[rustfmt::skip]
    static SYSTEM_COLOURS: [(u8, u8, u8); 16] = [
        (  0,   0,   0), (205,   0,   0), (  0, 205,   0), (205, 205,   0),
        (  0,   0, 238), (205,   0, 205), (  0, 205, 205), (229, 229, 229),
        (127, 127, 127), (255,   0,   0), (  0, 255,   0), (255, 255,   0),
        ( 92,  92, 255), (255,   0, 255), (  0, 255, 255), (255, 255, 255),
    ];

    // System colours
    for (idx, rgb) in SYSTEM_COLOURS.iter().enumerate() {
        assert_eq!(*rgb, to_rgb(idx as u8));
    }

    // Colour cube
    for idx in 0..216 {
        assert_eq!(
            (
                CUBE_VALUES[idx / 36],
                CUBE_VALUES[(idx / 6) % 6],
                CUBE_VALUES[idx % 6]
            ),
            to_rgb(16 + idx as u8)
        );
    }

    // Greyscale ramp
    for idx in 0..24 {
        let y = idx * 10 + 8;
        assert_eq!((y, y, y), to_rgb(idx + 232));
    }
}

/// Tries all colours in the 256-colour ANSI palette and chooses one with
/// smallest ΔE*₀₀ to `rgb(y, y, y)`.
fn best_grey(y: u8) -> u8 {
    let grey = [y, y, y];
    CUBE_VALUES
        .iter()
        .enumerate()
        .map(|(idx, v)| (*v, idx as u8 * (36 + 6 + 1) + 16))
        .chain((0..24u8).map(|idx| (idx * 10 + 8, idx + 232)))
        .fold((f32::INFINITY, 0), |best, elem| {
            let d = empfindung::de2000::diff_rgb(&grey, &[elem.0, elem.0, elem.0]);
            if d < best.0 {
                (d, elem.1)
            } else {
                best
            }
        })
        .1
}

#[test]
fn test_from_rgb_grey() {
    for i in 0..256 {
        assert_eq!(best_grey(i as u8), to_ansi((i as u8, i as u8, i as u8)));
    }
}

#[test]
fn test_to_ansi_exact() {
    for i in 16..256 {
        let rgb = to_rgb(i as u8);
        let got = to_ansi(rgb);
        assert_eq!(i as u8, got, "want {:?} but got {:?}", rgb, to_rgb(got));
    }
}

#[test]
#[rustfmt::skip]
fn test_to_ansi_approx() {
    assert_eq!( 16, to_ansi((  1,   1,   1)));
    assert_eq!(232, to_ansi((  7,   7,   7)));
    assert_eq!(232, to_ansi((  8,   7,   8)));
    assert_eq!( 64, to_ansi(( 97, 134,   8)));
}
