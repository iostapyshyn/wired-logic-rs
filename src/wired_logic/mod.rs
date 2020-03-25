extern crate image;
mod parser;

use image::RgbaImage;

pub const MAX_CHARGE: u8 = 6;
pub const VOID: image::Rgba<u8> = image::Rgba([0x00, 0x00, 0x00, 0xff]);
pub const CHARGE: [image::Rgba<u8>; (MAX_CHARGE + 1) as usize] = [
    image::Rgba([0x88, 0x00, 0x00, 0xff]),
    image::Rgba([0xff, 0x00, 0x00, 0xff]),
    image::Rgba([0xff, 0x22, 0x00, 0xff]),
    image::Rgba([0xff, 0x44, 0x00, 0xff]),
    image::Rgba([0xff, 0x66, 0x00, 0xff]),
    image::Rgba([0xff, 0x88, 0x00, 0xff]),
    image::Rgba([0xff, 0xaa, 0x00, 0xff]),
];

// The struct contains indices of transistor pins in wires
// array of the Circuit.
struct Transistor {
    base: usize,
    pins: [usize; 2],
}

// Generic wire struct.
pub struct Wire {
    pub pixels: Vec<(u32, u32)>,
    is_source: bool,
    /* Indices of connected transistors.
     * Does not contain the ones that are connected by the base (control) pin. */
    transistors: Vec<usize>,
}

/** All transistors and wires live in a big pool called Circuit
 ** and are owned by the respective vectors. */
pub struct Circuit {
    pub bounds: (u32, u32),       // original image bounds
    pub wires: Vec<Wire>,         // wires pool
    pub state: Vec<u8>,           // current charges of the wires
    transistors: Vec<Transistor>, // transistors pool
}

impl Circuit {
    pub fn new(img: &RgbaImage) -> Self {
        let mut circuit = Circuit {
            bounds: img.dimensions(),
            wires: Vec::new(),
            transistors: Vec::new(),
            state: Vec::new(),
        };

        parser::parse(&img, &mut circuit);

        circuit
    }

    pub fn update(&mut self, img: &RgbaImage) {
        parser::parse(img, self);
    }

    fn transistors_of(&self, wire: usize) -> Vec<&Transistor> {
        self.wires[wire]
            .transistors
            .iter()
            .map(|i| &self.transistors[*i])
            .collect()
    }

    fn trace_source(&self, wire: usize) -> u8 {
        let mut source_charge = 0;
        for transistor in self.transistors_of(wire) {
            if self.state[transistor.base] == 0 {
                for pin in transistor.pins.iter() {
                    if *pin != wire && self.state[*pin] > source_charge {
                        if self.state[*pin] == MAX_CHARGE {
                            return MAX_CHARGE;
                        } else {
                            source_charge = self.state[*pin];
                        }
                    }
                }
            }
        }
        source_charge
    }

    pub fn step(&mut self) -> &Self {
        let mut new_state = self.state.clone();

        for (i, charge) in (0..self.wires.len()).zip(&mut new_state) {
            if self.wires[i].is_source && *charge < MAX_CHARGE {
                *charge += 1;
            } else if !self.wires[i].is_source {
                let source = self.trace_source(i);

                if source > *charge + 1 {
                    *charge += 1;
                } else if source <= *charge && *charge > 0 {
                    *charge -= 1;
                }
            }
        }

        self.state = new_state;

        self
    }

    pub fn export(&self, dest: &mut image::RgbaImage) {
        self.wires.iter().for_each(|wire| {
            wire.pixels.iter().for_each(|coord| {
                dest.put_pixel(coord.0, coord.1, CHARGE[0]);
            })
        });
    }

    pub fn render(&self, dest: &mut image::RgbaImage) {
        self.wires.iter().enumerate().for_each(|(i, wire)| {
            wire.pixels.iter().for_each(|coord| {
                dest.put_pixel(coord.0, coord.1, CHARGE[self.state[i] as usize]);
            })
        });
    }
}
