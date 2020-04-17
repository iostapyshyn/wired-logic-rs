extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;

mod wired_logic;
use wired_logic::render::RenderFrames;

#[wasm_bindgen]
pub enum Cell {
    Void,
    Wire,
}

impl Into<image::Rgba<u8>> for Cell {
    fn into(self) -> image::Rgba<u8> {
        match self {
            Cell::Void => wired_logic::VOID,
            Cell::Wire => wired_logic::CHARGE[0],
        }
    }
}

#[wasm_bindgen]
pub struct Circuit {
    circuit: wired_logic::Circuit,
    image: image::RgbaImage,
}

#[wasm_bindgen]
impl Circuit {
    pub fn new(data: &[u8]) -> Self {
        console_error_panic_hook::set_once();

        let image = image::load_from_memory(data)
            .unwrap()
            .as_rgba8()
            .unwrap()
            .to_owned();
        let circuit = wired_logic::Circuit::new(&image);

        Self { circuit, image }
    }

    pub fn tick(&mut self) {
        self.circuit.step();
        self.circuit.render(&mut self.image);
    }

    pub fn at(&self, x: u32, y: u32) -> Cell {
        if x >= self.image.width() || y >= self.image.height() {
            return Cell::Void;
        }

        for i in wired_logic::CHARGE.iter() {
            if self.image.get_pixel(x, y) == i {
                return Cell::Wire;
            }
        }

        Cell::Void
    }

    pub fn pixels_view(&mut self) -> js_sys::Uint8ClampedArray {
        unsafe { js_sys::Uint8ClampedArray::view(&self.image) }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, cell: Cell) {
        self.image.put_pixel(x, y, cell.into());
        self.circuit.update(&self.image);
    }

    pub fn draw_line(&mut self, start_x: f32, start_y: f32, end_x: f32, end_y: f32, cell: Cell) {
        imageproc::drawing::draw_line_segment_mut(
            &mut self.image,
            (start_x, start_y),
            (end_x, end_y),
            cell.into(),
        );

        self.circuit.update(&self.image);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, cell: Cell) {
        let rect = imageproc::rect::Rect::at(x, y).of_size(w, h);
        imageproc::drawing::draw_filled_rect_mut(&mut self.image, rect, cell.into());
        self.circuit.update(&self.image);
    }

    pub fn toggle_pixel(&mut self, x: u32, y: u32) {
        let cell = match self.at(x, y) {
            Cell::Wire => Cell::Void,
            Cell::Void => Cell::Wire,
        };

        self.draw_pixel(x, y, cell);
    }

    pub fn export(&mut self) -> js_sys::Uint8ClampedArray {
        let saved_state = self.circuit.state.clone();
        self.reset();

        let pixels =
            js_sys::Uint8ClampedArray::new(&wasm_bindgen::JsValue::from(self.pixels_view()));

        self.circuit.state = saved_state;
        self.circuit.render(&mut self.image);

        pixels
    }

    pub fn reset(&mut self) {
        for i in &mut self.circuit.state {
            *i = 0;
        }

        self.circuit.render(&mut self.image);
    }

    pub fn render_gif(&mut self, delay: u64) -> js_sys::Uint8Array {
        let frames = self
            .circuit
            .render_frames(&self.image, std::time::Duration::from_millis(delay));

        let mut buf = Vec::<u8>::new();

        {
            let mut encoder = image::gif::Encoder::new(&mut buf);
            encoder.encode_frames(frames).unwrap();
        }

        js_sys::Uint8Array::from(&buf[..])
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
}
