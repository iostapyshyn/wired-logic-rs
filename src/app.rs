extern crate image;
extern crate imageproc;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use image::RgbaImage;

use std::time;

use crate::wired_logic::*;

const TIMESTEP: usize = 10;
const CURSOR: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0xff, 0xff, 0xff, 0xff);

#[derive(Clone, Copy, PartialEq)]
enum State {
    Running,
    Paused,
    Changed,
    Drawing(bool), // true for drawing, false for erasing.
    Quit,
}

struct Canvas {
    scale: u32,
    tex: sdl2::render::Texture,
    ren: sdl2::render::Canvas<sdl2::video::Window>,
}

pub struct App {
    img: RgbaImage,
    state: State,
    delay: usize,
    circuit: Circuit,
    canvas: Canvas,
    event_pump: sdl2::EventPump,
    mouse: (i32, i32),
}

pub fn init(img: RgbaImage) -> App {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let bounds = img.dimensions();
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

    let circuit = Circuit::new(&img);
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
        canvas: Canvas { scale, ren, tex },
        state: State::Running,
        mouse: (0, 0),
        delay: 0,
        circuit,
        event_pump,
        img,
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

                KeyDown { keycode, .. } => match keycode {
                    Some(K) => self.delay += TIMESTEP,
                    Some(J) => self.delay = self.delay.saturating_sub(TIMESTEP),

                    Some(Space) => {
                        self.state = match self.state {
                            State::Running => State::Paused,
                            State::Paused => State::Running,
                            State::Changed => {
                                self.circuit = Circuit::new(&self.img);
                                State::Running
                            }
                            _ => self.state,
                        }
                    }

                    Some(Period) => self.circuit.step_and_render(&mut self.img),

                    Some(Return) => {
                        self.circuit.state.iter_mut().for_each(|i| *i = 0);
                        self.circuit.step_and_render(&mut self.img);
                        self.state = State::Paused;
                    }

                    _ => {}
                },

                MouseButtonDown { x, y, .. } => {
                    let scale = self.canvas.scale;

                    let mut drawing = true;
                    for i in 0..=MAX_CHARGE {
                        if *self.img.get_pixel(x as u32 / scale, y as u32 / scale)
                            == CHARGE[i as usize]
                        {
                            drawing = false;
                        }
                    }

                    self.img.put_pixel(
                        x as u32 / scale,
                        y as u32 / scale,
                        if drawing { CHARGE[0] } else { VOID },
                    );

                    self.state = State::Drawing(drawing);
                }

                MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    self.mouse = (x, y);
                    if let State::Drawing(drawing) = self.state {
                        let scale = self.canvas.scale as f32;

                        let start = (x as f32 / scale, y as f32 / scale);
                        let end = (start.0 - xrel as f32 / scale, start.1 - yrel as f32 / scale);

                        imageproc::drawing::draw_line_segment_mut(
                            &mut self.img,
                            start,
                            end,
                            if drawing { CHARGE[0] } else { VOID },
                        );
                    }
                }

                MouseButtonUp { .. } => {
                    self.state = State::Changed;
                }

                _ => {}
            }
        }
    }

    fn update(&mut self) {
        let status;
        let window = self.canvas.ren.window_mut();
        let _ = window.set_title(match self.state {
            State::Running => {
                status = format!("wired-rs: {}ms delay.", self.delay);
                &status
            }
            State::Paused => "wired-rs: paused.",
            State::Changed | State::Drawing(..) => "wired-rs: modified.",
            _ => "",
        });

        if self.state == State::Running {
            self.circuit.step_and_render(&mut self.img);
        }
    }

    fn draw(&mut self) {
        let canvas = &mut self.canvas;

        let pitch = (self.img.width() * 4) as usize;
        let _ = canvas.tex.update(None, &self.img, pitch);
        let _ = canvas.ren.copy(&canvas.tex, None, None);

        canvas.ren.set_draw_color(CURSOR);
        let cursor = Rect::new(
            self.mouse.0 / canvas.scale as i32 * canvas.scale as i32,
            self.mouse.1 / canvas.scale as i32 * canvas.scale as i32,
            canvas.scale,
            canvas.scale,
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
