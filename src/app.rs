extern crate image;
extern crate imageproc;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use sdl2::rect::Rect;

use image::RgbaImage;

use std::time;

use crate::wired_logic::*;

const TIMESTEP: u128 = 50;

mod colors {
    use sdl2::pixels::Color;
    pub const VOID: Color = Color::RGB(0x0a, 0x0a, 0x0a);
    pub const CURSOR: Color = Color::RGB(0xff, 0xff, 0xff);
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

#[derive(Clone, Copy, PartialEq)]
enum State {
    Running,
    Paused,
    Editing,
    Quit,
}

struct Canvas {
    scale: u32,
    img_tex: sdl2::render::Texture,
    ren: sdl2::render::Canvas<sdl2::video::Window>,
}

pub struct App {
    state: State,
    dragging: bool,
    erasing: bool,
    delay: u128,
    circuit: Circuit,
    canvas: Canvas,
    img: RgbaImage,
    event_pump: sdl2::EventPump,
}

pub fn init(title: &str, img: RgbaImage) -> App {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let scale = 2;

    let bounds = img.dimensions();
    let window = video_subsystem
        .window(title, bounds.0 * scale, bounds.1 * scale)
        .position_centered()
        .build()
        .unwrap();

    let ren = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    sdl_context.mouse().show_cursor(false);

    let mut img_tex = ren
        .texture_creator()
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGBA32, bounds.0, bounds.1)
        .unwrap();

    img_tex.update(None, &img, (bounds.0 * 4) as usize).unwrap();

    App {
        state: State::Running,
        circuit: Circuit::from_image(&img),
        erasing: false,
        dragging: false,
        delay: 0,
        event_pump,
        img,
        canvas: Canvas {
            scale,
            ren,
            img_tex,
        },
    }
}

impl App {
    fn eventpoll(&mut self) {
        use Event::*;
        use Keycode::*;
        for event in self.event_pump.poll_iter() {
            match event {
                Quit { .. }
                | KeyUp {
                    keycode: Some(Escape),
                    ..
                } => self.state = State::Quit,

                KeyDown {
                    keycode, keymod, ..
                } => match keycode {
                    Some(Equals) if keymod == Mod::LSHIFTMOD => self.delay += TIMESTEP,
                    Some(Minus) => {
                        if self.delay > TIMESTEP {
                            self.delay -= TIMESTEP
                        } else {
                            self.delay = 0
                        }
                    }
                    Some(Space) => {
                        self.state = match self.state {
                            State::Running => State::Paused,
                            State::Paused => State::Running,
                            State::Editing => {
                                self.circuit = Circuit::from_image(&self.img);
                                State::Running
                            }
                            _ => self.state,
                        }
                    }
                    _ => {}
                },

                MouseButtonDown { x, y, .. } => {
                    self.dragging = true;
                    self.state = State::Editing;

                    self.erasing = *self
                        .img
                        .get_pixel(x as u32 / self.canvas.scale, y as u32 / self.canvas.scale)
                        == image::Rgba([0x88, 0x00, 0x00, 0xff]);

                    let color = if self.erasing {
                        image::Rgba([0x00, 0x00, 0x00, 0xff])
                    } else {
                        image::Rgba([0x88, 0x00, 0x00, 0xff])
                    };

                    self.img.put_pixel(
                        x as u32 / self.canvas.scale,
                        y as u32 / self.canvas.scale,
                        color,
                    );
                }

                MouseMotion {
                    x, y, xrel, yrel, ..
                } if self.dragging => {
                    let color = if self.erasing {
                        image::Rgba([0x00, 0x00, 0x00, 0xff])
                    } else {
                        image::Rgba([0x88, 0x00, 0x00, 0xff])
                    };
                    let start = (
                        (x / self.canvas.scale as i32) as f32,
                        (y / self.canvas.scale as i32) as f32,
                    );
                    let end = (
                        ((x - xrel) / self.canvas.scale as i32) as f32,
                        ((y - yrel) / self.canvas.scale as i32) as f32,
                    );

                    imageproc::drawing::draw_line_segment_mut(&mut self.img, start, end, color);
                }

                MouseButtonUp { .. } => {
                    self.dragging = false;
                }

                _ => {}
            }
        }
    }

    fn update(&mut self) {
        match self.state {
            State::Running => self.circuit.step(),
            State::Editing => self
                .canvas
                .img_tex
                .update(None, &self.img, (self.circuit.bounds.0 * 4) as usize)
                .unwrap(),
            _ => {}
        }
    }

    fn draw_wire(canvas: &mut Canvas, (wire, state): (&Wire, &u8)) {
        canvas.ren.set_draw_color(colors::WIRE[*state as usize]);

        for pixel in &wire.pixels {
            canvas
                .ren
                .fill_rect(Rect::new(
                    (pixel.0 as u32 * canvas.scale) as i32,
                    (pixel.1 as u32 * canvas.scale) as i32,
                    canvas.scale,
                    canvas.scale,
                ))
                .unwrap();
        }
    }

    fn draw(&mut self) {
        let canvas = &mut self.canvas;

        canvas.ren.set_draw_color(colors::VOID);
        canvas.ren.clear();

        // Present the original image.
        canvas.ren.copy(&canvas.img_tex, None, None).unwrap();

        // Overlap the image with charge-colored wires (if not in editing mode).
        if self.state == State::Running {
            self.circuit
                .wires
                .iter()
                .zip(self.circuit.state.iter())
                .for_each(|wire| Self::draw_wire(canvas, wire));
        }

        canvas.ren.set_draw_color(colors::CURSOR);

        canvas
            .ren
            .fill_rect(Rect::new(
                self.event_pump.mouse_state().x() / canvas.scale as i32 * canvas.scale as i32,
                self.event_pump.mouse_state().y() / canvas.scale as i32 * canvas.scale as i32,
                canvas.scale as u32,
                canvas.scale as u32,
            ))
            .unwrap();

        canvas.ren.present();
    }

    pub fn run(mut self) {
        let mut last_time = time::Instant::now();

        while self.state != State::Quit {
            self.eventpoll();

            if last_time.elapsed().as_millis() > self.delay {
                last_time = time::Instant::now();
                self.update();
            }

            self.draw();
        }
    }
}
