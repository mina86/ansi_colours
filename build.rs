extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/ansi256.c")
        .compile("ansi256");
}
