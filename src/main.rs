extern crate image;

mod app;
pub mod wired_logic;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        let img = image::RgbaImage::from_pixel(100, 100, wired_logic::VOID);
        app::init(img).run();
    } else if args.len() == 2 {
        let img = image::open(&args[1]).unwrap();
        app::init(img.to_rgba()).run();
    } else if args.len() == 3 {
        // Implement GIF rendering;
        unimplemented!();
    } else {
        eprintln!("USAGE: {} [input [output.gif]]", args[0]);
        std::process::exit(1);
    }
}
