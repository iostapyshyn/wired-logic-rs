extern crate sdl2;
mod wired_logic;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time;

use wired_logic::*;

const CELLSIZE: usize = 2;

mod colors {
    use sdl2::pixels::Color;
    pub const VOID: Color = Color::RGB(0x0a, 0x0a, 0x0a);
    pub const WIRE: [Color; 7] = [
        Color::RGB(0x88, 0x00, 0x00),
        Color::RGB(0xff, 0x00, 0x00),
        Color::RGB(0xff, 0x22, 0x00),
        Color::RGB(0xff, 0x44, 0x00),
        Color::RGB(0xff, 0x66, 0x00),
        Color::RGB(0xff, 0x88, 0x00),
        Color::RGB(0xff, 0xaa, 0x00),
    ];
}

struct AppState {
    running: bool,
    circuit: Circuit,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
}

impl AppState {
    fn init(title: &str, circuit: Circuit) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                title,
                (circuit.bounds.0 * CELLSIZE) as u32,
                (circuit.bounds.1 * CELLSIZE) as u32,
            )
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        AppState {
            running: true,
            circuit,
            canvas,
            event_pump,
        }
    }

    fn eventpoll(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,
                _ => {}
            }
        }
    }

    fn update(&mut self) {
        self.circuit.step();
    }

    fn draw(&mut self) {
        self.canvas.set_draw_color(colors::VOID);
        self.canvas.clear();

        for i in 0..self.circuit.state.len() {
            self.canvas
                .set_draw_color(colors::WIRE[self.circuit.state[i] as usize]);

            for pixel in &self.circuit.wires[i].pixels {
                self.canvas
                    .fill_rect(sdl2::rect::Rect::new(
                        (pixel.0 * CELLSIZE) as i32,
                        (pixel.1 * CELLSIZE) as i32,
                        CELLSIZE as u32,
                        CELLSIZE as u32,
                    ))
                    .unwrap();
            }
        }

        self.canvas.present();
    }
}

pub fn main() {
    let circuit = Circuit::from_image("input.gif").unwrap();
    let mut g = AppState::init("wired-rs", circuit);

    let mut last_time = time::Instant::now();

    while g.running {
        g.eventpoll();

        if last_time.elapsed().as_millis() > 0 {
            last_time = time::Instant::now();
            g.update();
        }

        g.draw();
    }
}
