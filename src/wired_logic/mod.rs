extern crate image;
mod parser;

use image::GenericImageView;

const MAX_CHARGE: u8 = 6;

// The struct contains indices of transistor pins in wires
// array of the Circuit.
pub struct Transistor {
    pub base: usize,
    pub pins: [usize; 2],
    pub position: (usize, usize),
}

// Generic wire struct.
pub struct Wire {
    pub is_source: bool,
    /* Indices of connected transistors.
     * Does not contain the ones that are connected by the base (control) pin. */
    pub transistors: Vec<usize>,
    pub pixels: Vec<(usize, usize)>,
}

/** All transistors and wires live in a big pool called Circuit
 ** and are owned by the respective vectors. */
pub struct Circuit {
    pub bounds: (usize, usize),       // original image bounds
    pub wires: Vec<Wire>,             // wires pool
    pub transistors: Vec<Transistor>, // transistors pool
    pub state: Vec<u8>,               // current charges of the wires
}

fn rgba_distance(one: image::Rgba<u8>, other: image::Rgba<u8>) -> usize {
    let (r, g, b, a) = (
        (one[0] as i16 - other[0] as i16).abs(),
        (one[1] as i16 - other[1] as i16).abs(),
        (one[2] as i16 - other[2] as i16).abs(),
        (one[3] as i16 - other[3] as i16).abs(),
    );
    (r + g + b + a) as usize
}

impl Circuit {
    pub fn from_image(filename: &str) -> Result<Self, image::ImageError> {
        const RED_THRESHOLD: usize = 50;

        let img = image::open(filename)?;
        let bounds = img.dimensions();

        let wire: Vec<bool> = img
            .pixels()
            .map(|i| {
                if rgba_distance(i.2, image::Rgba([0x88, 0x00, 0x00, 0xff])) < RED_THRESHOLD {
                    true
                } else {
                    false
                }
            })
            .collect();

        Ok(Self::new(&wire, (bounds.0 as usize, bounds.1 as usize)))
    }

    pub fn new(wire: &[bool], bounds: (usize, usize)) -> Circuit {
        parser::parse(&wire, bounds)
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

    pub fn step(&mut self) {
        let mut state = self.state.clone();

        (0..self.wires.len())
            .into_iter()
            .zip(&mut state)
            .for_each(|(i, charge)| {
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
            });

        self.state = state;
    }
}
