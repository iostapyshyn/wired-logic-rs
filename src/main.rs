extern crate image;

mod app;
pub mod wired_logic;

fn main() {
    let filename = if std::env::args().len() != 2 {
        eprintln!("Usage: {} filename", std::env::args().nth(0).unwrap());
        std::process::exit(1)
    } else {
        std::env::args().last().unwrap()
    };

    let img = image::open(filename).unwrap();
    let g = app::init("wired-rs", img.to_rgba());

    g.run();
}
