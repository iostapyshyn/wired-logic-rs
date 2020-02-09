extern crate image;

use image::GenericImageView;

const MAX_CHARGE: u8 = 6;

pub struct Transistor {
    pub base: usize,
    pub pins: [usize; 2],
    pub position: (u32, u32),
}

pub struct Wire {
    pub is_source: bool,
    pub charge: u8,
    charge_new: u8,
    pub transistors: Vec<usize>,
    pub pixels: Vec<(u32, u32)>,
}

impl Wire {
    fn new() -> Self {
        Self {
            is_source: false,
            charge: 0,
            charge_new: 0,
            transistors: Vec::new(),
            pixels: Vec::new(),
        }
    }
}

/** All transistors and wires live in a big pool called Circuit
 ** and are owned by the respective vectors. */
pub struct Circuit {
    pub bounds: (u32, u32),
    pub wires: Vec<Wire>,
    pub transistors: Vec<Transistor>,
}

#[derive(Copy, Clone, PartialEq)]
enum Cell {
    Wire(Option<usize>),
    Transistor,
    Void,
}

fn neighbours(coord: (u32, u32)) -> [(u32, u32); 4] {
    [
        (coord.0 - 1, coord.1),
        (coord.0 + 1, coord.1),
        (coord.0, coord.1 - 1),
        (coord.0, coord.1 + 1),
    ]
}

fn is_source(coord: (u32, u32), pitch: u32, pixels: &[Cell]) -> bool {
    for i in [
        (coord.0 + 1, coord.1),     // right
        (coord.0, coord.1 + 1),     // down
        (coord.0 + 1, coord.1 + 1), // right and down
    ]
    .iter()
    {
        if let Some(pixel) = pixels.get((i.1 * pitch + i.0) as usize) {
            match pixel {
                Cell::Void | Cell::Transistor => return false,
                Cell::Wire(_) => continue,
            }
        }
    }

    true
}

fn count_neighbours(coord: (u32, u32), pitch: u32, pixels: &[Cell]) -> u32 {
    let mut acc = 0;
    for i in neighbours(coord).iter() {
        if let Some(pixel) = pixels.get((i.1 * pitch + i.0) as usize) {
            match pixel {
                Cell::Wire(_) => acc += 1,
                _ => {}
            }
        }
    }

    acc
}

fn bucket_wire(coord: (u32, u32), pixels: &mut [Cell], circuit: &mut Circuit) {
    fn sub(coord: (u32, u32), parent: (u32, u32)) -> (i32, i32) {
        (
            (coord.0 as i32 - parent.0 as i32),
            (coord.1 as i32 - parent.1 as i32),
        )
    }

    let pitch = circuit.bounds.0;

    let wire_index = circuit.wires.len();
    circuit.wires.push(Wire::new());

    let mut stack = vec![(coord, coord)];
    while let Some((coord, parent)) = stack.pop() {
        match pixels[(coord.1 * pitch + coord.0) as usize] {
            Cell::Void => match count_neighbours(coord, pitch, pixels) {
                4 => {
                    let vec = sub(coord, parent);
                    let jump = (
                        (coord.0 as i32 + vec.0) as u32,
                        (coord.1 as i32 + vec.1) as u32,
                    );
                    stack.push((jump, coord));
                }
                3 => pixels[(coord.1 * pitch + coord.0) as usize] = Cell::Transistor,
                _ => continue,
            },
            Cell::Wire(None) => {
                if is_source(coord, pitch, pixels) {
                    circuit.wires[wire_index].is_source = true;
                }

                circuit.wires[wire_index].pixels.push(coord);

                pixels[(coord.1 * pitch + coord.0) as usize] = Cell::Wire(Some(wire_index));

                for neighbour in neighbours(coord).iter() {
                    stack.push((*neighbour, coord));
                }
            }
            Cell::Wire(Some(_)) | Cell::Transistor => {}
        }
    }
}

fn add_transistor(coord: (u32, u32), pixels: &mut Vec<Cell>, circuit: &mut Circuit) {
    enum Dir {
        Up = 0,
        Down = 1,
        Left = 2,
        Right = 3,
    };

    let mut dirs = [Option::<usize>::None; 4];
    if let Some(Cell::Wire(Some(i))) =
        pixels.get(((coord.1 - 1) * circuit.bounds.0 + coord.0) as usize)
    {
        dirs[Dir::Up as usize] = Some(*i);
    }
    if let Some(Cell::Wire(Some(i))) =
        pixels.get(((coord.1 + 1) * circuit.bounds.0 + coord.0) as usize)
    {
        dirs[Dir::Down as usize] = Some(*i);
    }
    if let Some(Cell::Wire(Some(i))) =
        pixels.get((coord.1 * circuit.bounds.0 + (coord.0 - 1)) as usize)
    {
        dirs[Dir::Left as usize] = Some(*i);
    }
    if let Some(Cell::Wire(Some(i))) =
        pixels.get((coord.1 * circuit.bounds.0 + (coord.0 + 1)) as usize)
    {
        dirs[Dir::Right as usize] = Some(*i);
    }

    let mut transistor = Transistor {
        pins: [0; 2],
        base: 0,
        position: coord,
    };

    if dirs[Dir::Up as usize].is_some() && dirs[Dir::Down as usize].is_some() {
        transistor.pins = [
            dirs[Dir::Up as usize].unwrap(),
            dirs[Dir::Down as usize].unwrap(),
        ];
        transistor.base = if dirs[Dir::Left as usize].is_some() {
            dirs[Dir::Left as usize].unwrap()
        } else if dirs[Dir::Right as usize].is_some() {
            dirs[Dir::Right as usize].unwrap()
        } else {
            panic!();
        }
    }

    if dirs[Dir::Left as usize].is_some() && dirs[Dir::Right as usize].is_some() {
        transistor.pins = [
            dirs[Dir::Left as usize].unwrap(),
            dirs[Dir::Right as usize].unwrap(),
        ];
        transistor.base = if dirs[Dir::Up as usize].is_some() {
            dirs[Dir::Up as usize].unwrap()
        } else if dirs[Dir::Down as usize].is_some() {
            dirs[Dir::Down as usize].unwrap()
        } else {
            panic!();
        }
    }

    if transistor.pins[0] == transistor.base || transistor.pins[1] == transistor.base {
        return;
    }

    circuit.wires[transistor.pins[0]]
        .transistors
        .push(circuit.transistors.len());
    circuit.wires[transistor.pins[1]]
        .transistors
        .push(circuit.transistors.len());
    circuit.transistors.push(transistor);
}

impl Circuit {
    pub fn new(filename: &str) -> Result<Self, image::ImageError> {
        let img = image::open(filename)?;

        let bounds = img.dimensions();

        let mut states = vec![Cell::Void; (bounds.0 * bounds.1) as usize];
        for i in img.pixels() {
            if i.2 == image::Rgba::<u8>([0x88, 0x00, 0x00, 0xff]) {
                states[(i.1 * bounds.0 + i.0) as usize] = Cell::Wire(None);
            }
        }

        let mut circuit = Circuit {
            bounds,
            wires: Vec::new(),
            transistors: Vec::new(),
        };

        /* First pass: wires. */
        for y in 0..bounds.1 {
            for x in 0..bounds.0 {
                if let Cell::Wire(None) = states[(y * bounds.0 + x) as usize] {
                    bucket_wire((x, y), &mut states, &mut circuit);
                }
            }
        }

        /* Second pass: transistors. */
        for y in 0..bounds.1 {
            for x in 0..bounds.0 {
                if let Cell::Transistor = states[(y * bounds.0 + x) as usize] {
                    add_transistor((x, y), &mut states, &mut circuit);
                }
            }
        }

        Ok(circuit)
    }

    pub fn step(&mut self) {
        for wire in 0..self.wires.len() {
            if self.wires[wire].is_source && self.wires[wire].charge < MAX_CHARGE {
                self.wires[wire].charge_new = self.wires[wire].charge + 1;
            } else if !self.wires[wire].is_source {
                self.wires[wire].charge_new = self.wires[wire].charge;

                let mut source = 0;
                for transistor in 0..self.wires[wire].transistors.len() {
                    let transistor = self.wires[wire].transistors[transistor];
                    let transistor = &self.transistors[transistor];

                    if transistor.base != wire && self.wires[transistor.base].charge == 0 {
                        for pin in transistor.pins.iter() {
                            if *pin != wire && self.wires[*pin].charge > source {
                                source = self.wires[*pin].charge;
                            }
                        }
                    }
                }

                if source > self.wires[wire].charge + 1 {
                    self.wires[wire].charge_new += 1;
                } else if source <= self.wires[wire].charge && self.wires[wire].charge > 0 {
                    self.wires[wire].charge_new -= 1;
                }
            }
        }

        for wire in &mut self.wires {
            wire.charge = wire.charge_new;
        }
    }
}
