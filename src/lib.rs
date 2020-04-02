use wasm_bindgen::prelude::*;

mod wired_logic;

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
    source: image::RgbaImage,
}

#[wasm_bindgen]
impl Circuit {
    pub fn new(data: &[u8]) -> Self {
        let source = image::load_from_memory(data)
            .unwrap()
            .as_rgba8()
            .unwrap()
            .to_owned();
        let circuit = wired_logic::Circuit::new(&source);

        Self { circuit, source }
    }

    pub fn tick(&mut self) {
        self.circuit.step();
        self.circuit.render(&mut self.source);
    }

    pub fn at(&self, x: u32, y: u32) -> Cell {
        for i in wired_logic::CHARGE.iter() {
            if self.source.get_pixel(x, y) == i {
                return Cell::Wire;
            }
        }

        Cell::Void
    }

    pub fn pixels_view(&mut self) -> js_sys::Uint8ClampedArray {
        unsafe { js_sys::Uint8ClampedArray::view(&self.source) }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, cell: Cell) {
        self.source.put_pixel(x, y, cell.into());
        self.circuit.update(&self.source);
    }

    pub fn draw_line(&mut self, start_x: f32, start_y: f32, end_x: f32, end_y: f32, cell: Cell) {
        imageproc::drawing::draw_line_segment_mut(
            &mut self.source,
            (start_x, start_y),
            (end_x, end_y),
            cell.into(),
        );

        self.circuit.update(&self.source);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, cell: Cell) {
        let rect = imageproc::rect::Rect::at(x, y).of_size(w, h);
        imageproc::drawing::draw_filled_rect_mut(&mut self.source, rect, cell.into());
        self.circuit.update(&self.source);
    }

    pub fn toggle_pixel(&mut self, x: u32, y: u32) {
        let cell = match self.at(x, y) {
            Cell::Wire => Cell::Void,
            Cell::Void => Cell::Wire,
        };

        self.draw_pixel(x, y, cell);
    }

    /// Toggles the line based on the starting pixel;
    pub fn toggle_line(&mut self, start_x: u32, start_y: u32, end_x: u32, end_y: u32) {
        let cell = match self.at(start_x, start_y) {
            Cell::Wire => Cell::Void,
            Cell::Void => Cell::Wire,
        };

        self.draw_line(
            start_x as f32,
            start_y as f32,
            end_x as f32,
            end_y as f32,
            cell,
        );
    }

    pub fn width(&self) -> u32 {
        self.source.width()
    }

    pub fn height(&self) -> u32 {
        self.source.height()
    }
}
