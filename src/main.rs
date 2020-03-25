extern crate image;

mod app;
pub mod wired_logic;

use std::str::FromStr;

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        app::init(image::RgbaImage::from_pixel(160, 120, wired_logic::VOID)).run();
    } else if args.len() == 2 {
        let img = image::open(&args[1]);
        let img = if img.is_err() {
            if let Some((w, h)) = parse_pair(&args[1], 'x') {
                image::RgbaImage::from_pixel(w, h, wired_logic::VOID)
            } else {
                eprintln!("Unable to parse dimensions.");
                std::process::exit(1);
            }
        } else {
            img.unwrap().as_rgba8().unwrap().to_owned()
        };
        app::init(img).run();
    } else if args.len() == 3 {
        // Implement GIF rendering;
        unimplemented!();
    } else {
        eprintln!("USAGE: {} [input [output.gif] | WIDTHxHEIGHT]", args[0]);
        std::process::exit(1);
    }
}
