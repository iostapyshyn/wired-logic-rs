use wasm_bindgen::prelude::*;

mod wired_logic;
use wired_logic::*;

#[wasm_bindgen]
pub struct App {
    circuit: Circuit,
    source: image::RgbaImage,
}

#[wasm_bindgen]
impl App {
    pub fn new(data: &[u8]) -> Self {
        let source = image::load_from_memory(data)
            .unwrap()
            .as_rgba8()
            .unwrap()
            .to_owned();
        let circuit = Circuit::new(&source);

        Self { circuit, source }
    }

    pub fn tick(&mut self) {
        self.circuit.step();
    }

    pub fn render(&mut self) -> js_sys::Uint8ClampedArray {
        self.circuit.render(&mut self.source);

        unsafe { js_sys::Uint8ClampedArray::view(&self.source) }
    }

    pub fn width(&self) -> u32 {
        self.source.width()
    }

    pub fn height(&self) -> u32 {
        self.source.height()
    }
}
