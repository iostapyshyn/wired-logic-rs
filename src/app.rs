extern crate image;
extern crate imageproc;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use sdl2::EventPump;

use image::RgbaImage;

use std::time;

use crate::wired_logic::*;

const TIMESTEP: usize = 10;
const CURSOR: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0xff, 0xff, 0xff, 0xff);
const GRAY: image::Rgba<u8> = image::Rgba::<u8>([0x44, 0x44, 0x44, 0xff]);

#[derive(Clone, Copy, PartialEq)]
enum State {
    Running,
    Paused,
    Changed,
    Quit,
}

struct Canvas {
    tex: sdl2::render::Texture,
    ren: sdl2::render::Canvas<sdl2::video::Window>,
}

pub struct App {
    state: State,
    canvas: Canvas,
    image: RgbaImage,
    circuit: Circuit,
    event_pump: EventPump,
    cursor: (u32, u32),
    drawing: bool,
    scale: u32,
    delay: usize,
}

pub fn init(image: RgbaImage) -> App {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let bounds = image.dimensions();
    let dm = video_subsystem.current_display_mode(0).unwrap();
    let scale = std::cmp::min(
        4 * dm.w as u32 / 5 / bounds.0,
        4 * dm.h as u32 / 5 / bounds.1,
    );

    let window = video_subsystem
        .window("", bounds.0 * scale, bounds.1 * scale)
        .position_centered()
        .build()
        .unwrap();

    let ren = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    sdl_context.mouse().show_cursor(false);

    let circuit = Circuit::new(&image);
    let tex = ren
        .texture_creator()
        .create_texture(
            sdl2::pixels::PixelFormatEnum::RGBA32,
            sdl2::render::TextureAccess::Streaming,
            bounds.0,
            bounds.1,
        )
        .unwrap();

    App {
        canvas: Canvas { ren, tex },
        state: State::Running,
        cursor: (0, 0),
        delay: 0,
        drawing: false,
        scale,
        circuit,
        event_pump,
        image,
    }
}

impl App {
    fn eventpoll(&mut self) {
        use Event::*;
        use Keycode::*;
        while let Some(event) = self.event_pump.poll_event() {
            match event {
                Quit { .. }
                | KeyUp {
                    keycode: Some(Escape),
                    ..
                } => self.state = State::Quit,

                KeyDown { keycode, .. } => match keycode {
                    Some(K) => self.delay += TIMESTEP,
                    Some(J) => self.delay = self.delay.saturating_sub(TIMESTEP),

                    Some(Space) => {
                        self.state = match self.state {
                            State::Running => State::Paused,
                            State::Paused => State::Running,
                            State::Changed => {
                                self.reload();
                                State::Running
                            }
                            _ => self.state,
                        }
                    }

                    Some(Period) => {
                        if self.state == State::Changed {
                            self.reload();
                        }

                        self.circuit.step().render(&mut self.image)
                    }

                    Some(Comma) => {
                        if self.state == State::Changed {
                            self.reload();
                        }

                        self.circuit.state.iter_mut().for_each(|i| *i = 0);
                        self.circuit.step().render(&mut self.image);
                        self.state = State::Paused;
                    }

                    Some(W) => {
                        if self.state == State::Changed {
                            self.reload();
                        }

                        self.circuit.export(&mut self.image);
                        self.image.save("wired-rs.png").unwrap();
                        self.circuit.render(&mut self.image);
                    }

                    Some(Right) => self.cursor.0 += 1,
                    Some(Left) => self.cursor.0 -= 1,
                    Some(Down) => self.cursor.1 += 1,
                    Some(Up) => self.cursor.1 -= 1,

                    _ => {}
                },

                MouseButtonDown { .. } => {
                    self.drawing = true;

                    for i in CHARGE.iter() {
                        if self.image.get_pixel(self.cursor.0, self.cursor.1) == i {
                            self.drawing = false;
                        }
                    }

                    self.image.put_pixel(
                        self.cursor.0,
                        self.cursor.1,
                        if self.drawing { CHARGE[0] } else { VOID },
                    );

                    self.state = State::Changed;
                }

                MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    self.cursor = (x as u32 / self.scale, y as u32 / self.scale);
                    if self.event_pump.mouse_state().left() {
                        let scale = self.scale as f32;
                        let (x, y) = (x as f32 / scale, y as f32 / scale);

                        imageproc::drawing::draw_line_segment_mut(
                            &mut self.image,
                            (x, y),
                            (x - xrel as f32 / scale, y - yrel as f32 / scale),
                            if self.drawing { CHARGE[0] } else { VOID },
                        );
                    }
                }

                _ => {}
            }
        }

        let keyboard_state = self.event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(Scancode::Tab) {
            self.image.put_pixel(self.cursor.0, self.cursor.1, GRAY);
            self.state = State::Changed;
        } else if keyboard_state.is_scancode_pressed(Scancode::LAlt) {
            self.image.put_pixel(self.cursor.0, self.cursor.1, VOID);
            self.state = State::Changed;
        } else if keyboard_state.is_scancode_pressed(Scancode::LShift) {
            self.image
                .put_pixel(self.cursor.0, self.cursor.1, CHARGE[0]);
            self.state = State::Changed;
        }
    }

    fn reload(&mut self) {
        self.circuit.update(&self.image);
        self.state = State::Paused;
    }

    fn update(&mut self) {
        let status;
        let window = self.canvas.ren.window_mut();
        let _ = window.set_title(match self.state {
            State::Running => {
                status = format!("wired-rs: {}ms delay.", self.delay);
                &status
            }
            State::Paused => "wired-rs: simulation on pause.",
            State::Changed => "wired-rs: modified..",
            _ => "",
        });

        if self.state == State::Running {
            self.circuit.step().render(&mut self.image);
        }
    }

    fn draw(&mut self) {
        let canvas = &mut self.canvas;

        let pitch = (self.image.width() * 4) as usize;
        let _ = canvas.tex.update(None, &self.image, pitch);
        let _ = canvas.ren.copy(&canvas.tex, None, None);

        canvas.ren.set_draw_color(CURSOR);
        let cursor = Rect::new(
            (self.cursor.0 * self.scale) as i32,
            (self.cursor.1 * self.scale) as i32,
            self.scale,
            self.scale,
        );
        let _ = canvas.ren.fill_rect(cursor);

        canvas.ren.present();
    }

    pub fn run(mut self) {
        let mut last_time = time::Instant::now();

        while self.state != State::Quit {
            self.eventpoll();

            if last_time.elapsed().as_millis() > self.delay as u128 {
                last_time = time::Instant::now();
                self.update();
            }

            self.draw();
        }
    }
}
