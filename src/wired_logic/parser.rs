use crate::wired_logic::*;

/// Returns Von Neumann neighbourhood coordinates as an array [up, down, left, right].
fn neighbourhood_neumann(coord: (u32, u32)) -> [(u32, u32); 4] {
    [
        (coord.0.wrapping_sub(1), coord.1),
        (coord.0.wrapping_add(1), coord.1),
        (coord.0, coord.1.wrapping_sub(1)),
        (coord.0, coord.1.wrapping_add(1)),
    ]
}

enum NeumannIndices {
    Up = 0,
    Down,
    Left,
    Right,
}

/// Returns diagonal neighbourhood coordinates as an array [up-left, up-right, down-left, down-right].
fn neighbourhood_diagonal(coord: (u32, u32)) -> [(u32, u32); 4] {
    [
        (coord.0.wrapping_sub(1), coord.1.wrapping_sub(1)),
        (coord.0.wrapping_sub(1), coord.1.wrapping_add(1)),
        (coord.0.wrapping_add(1), coord.1.wrapping_sub(1)),
        (coord.0.wrapping_add(1), coord.1.wrapping_add(1)),
    ]
}

enum DiagonalIndices {
    UpLeft = 0,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Copy, Clone, PartialEq)]
enum Pixel {
    Wire(u8, Option<usize>),
    Transistor,
    Void,
}

struct Parser<'a> {
    circuit: &'a mut Circuit,
    pixels: Vec<Pixel>,
    transistors: Vec<(u32, u32)>,
}

impl Parser<'_> {
    fn get(&self, coord: (u32, u32)) -> Pixel {
        // The wrapping hack should work up to usize max number of pixels
        // Since such screens are not common and are unlikely to be used by the end user:
        *self
            .pixels
            .get(
                coord
                    .1
                    .wrapping_mul(self.circuit.bounds.0)
                    .wrapping_add(coord.0) as usize,
            )
            .unwrap_or(&Pixel::Void)
    }

    fn set(&mut self, coord: (u32, u32), pixel: Pixel) {
        self.pixels[(coord.1 * self.circuit.bounds.0 + coord.0) as usize] = pixel
    }

    fn add_wire(&mut self, coord: (u32, u32)) -> u8 {
        let wire_index = self.circuit.wires.len();
        let mut wire_charge = 0;
        self.circuit.wires.push(Wire {
            is_source: false,
            transistors: Vec::new(),
            pixels: Vec::new(),
        });

        let mut stack = vec![(coord, coord)];
        while let Some((coord, parent)) = stack.pop() {
            match self.get(coord) {
                Pixel::Wire(charge, None) => {
                    if self.is_source(coord) {
                        self.circuit.wires[wire_index].is_source = true;
                        wire_charge = MAX_CHARGE;
                    }

                    if charge > wire_charge {
                        wire_charge = charge;
                    }

                    self.circuit.wires[wire_index].pixels.push(coord);
                    self.set(coord, Pixel::Wire(charge, Some(wire_index)));

                    neighbourhood_neumann(coord)
                        .iter()
                        .for_each(|i| stack.push((*i, coord)));
                }
                Pixel::Void => {
                    if self.is_crossing(coord) {
                        /* The current coordinate + difference from the previous cell
                         * should take us to the opposite side of the crossing. */
                        let jump = (
                            (coord.0 as i32 + (coord.0 as i32 - parent.0 as i32)) as u32,
                            (coord.1 as i32 + (coord.1 as i32 - parent.1 as i32)) as u32,
                        );
                        stack.push((jump, coord));
                    } else if self.is_transistor(coord) {
                        /* After assigning all the wires we need to pass once again,
                         * assigning all the transistors its relevant pins. */
                        self.transistors.push(coord);
                        self.set(coord, Pixel::Transistor);
                    }
                }

                _ => {}
            }
        }

        wire_charge
    }

    fn add_transistor(&mut self, coord: (u32, u32)) {
        use NeumannIndices::*;
        let (up, down, left, right) = (Up as usize, Down as usize, Left as usize, Right as usize);

        let mut neighbours = [Option::<usize>::None; 4];
        for (i, coord) in neighbourhood_neumann(coord).iter().enumerate() {
            if let Pixel::Wire(_, Some(wire)) = self.get(*coord) {
                neighbours[i] = Some(wire);
            }
        }

        let mut transistor = Transistor {
            pins: [0; 2],
            base: 0,
        };

        if neighbours[up].is_some() && neighbours[down].is_some() {
            transistor.pins = [neighbours[up].unwrap(), neighbours[down].unwrap()];
            transistor.base = if neighbours[left].is_some() {
                neighbours[left].unwrap()
            } else if neighbours[right].is_some() {
                neighbours[right].unwrap()
            } else {
                panic!();
            }
        }

        if neighbours[left].is_some() && neighbours[right].is_some() {
            transistor.pins = [neighbours[left].unwrap(), neighbours[right].unwrap()];
            transistor.base = if neighbours[up].is_some() {
                neighbours[up].unwrap()
            } else if neighbours[down].is_some() {
                neighbours[down].unwrap()
            } else {
                panic!();
            }
        }

        self.circuit.wires[transistor.pins[0]]
            .transistors
            .push(self.circuit.transistors.len());
        self.circuit.wires[transistor.pins[1]]
            .transistors
            .push(self.circuit.transistors.len());
        self.circuit.transistors.push(transistor);
    }

    fn is_source(&self, coord: (u32, u32)) -> bool {
        /* If cells to the right, down and down-right are occupied by wires,
         * we've got ourselves a square which is a power source. */
        for i in [
            (coord.0 + 1, coord.1),     // right
            (coord.0, coord.1 + 1),     // down
            (coord.0 + 1, coord.1 + 1), // right and down
        ]
        .iter()
        {
            match self.get(*i) {
                Pixel::Void | Pixel::Transistor => return false,
                Pixel::Wire(..) => continue,
            }
        }

        true
    }

    fn is_crossing(&self, coord: (u32, u32)) -> bool {
        for i in neighbourhood_diagonal(coord).iter() {
            if self.get(*i) != Pixel::Void {
                return false;
            }
        }

        for i in neighbourhood_neumann(coord).iter() {
            if self.get(*i) == Pixel::Void {
                return false;
            }
        }

        true
    }

    fn is_transistor(&self, coord: (u32, u32)) -> bool {
        use DiagonalIndices::*;
        use NeumannIndices::*;

        let neighbours_neumann: Vec<bool> = neighbourhood_neumann(coord)
            .iter()
            .map(|i| {
                if let Pixel::Wire(..) = self.get(*i) {
                    true
                } else {
                    false
                }
            })
            .collect();
        let neighbours_diagonal: Vec<bool> = neighbourhood_diagonal(coord)
            .iter()
            .map(|i| {
                if let Pixel::Wire(..) = self.get(*i) {
                    true
                } else {
                    false
                }
            })
            .collect();

        if (neighbours_neumann[Up as usize]
            && neighbours_neumann[Down as usize]
            && neighbours_neumann[Left as usize]
            && !neighbours_neumann[Right as usize]
            && !neighbours_diagonal[UpLeft as usize]
            && !neighbours_diagonal[DownLeft as usize])
            || (neighbours_neumann[Up as usize]
                && neighbours_neumann[Down as usize]
                && neighbours_neumann[Right as usize]
                && !neighbours_neumann[Left as usize]
                && !neighbours_diagonal[UpRight as usize]
                && !neighbours_diagonal[DownRight as usize])
            || (neighbours_neumann[Left as usize]
                && neighbours_neumann[Right as usize]
                && neighbours_neumann[Up as usize]
                && !neighbours_neumann[Down as usize]
                && !neighbours_diagonal[UpLeft as usize]
                && !neighbours_diagonal[UpRight as usize])
            || (neighbours_neumann[Left as usize]
                && neighbours_neumann[Right as usize]
                && neighbours_neumann[Down as usize]
                && !neighbours_neumann[Up as usize]
                && !neighbours_diagonal[DownLeft as usize]
                && !neighbours_diagonal[DownRight as usize])
        {
            return true;
        }

        false
    }
}

pub fn parse(img: &image::RgbaImage, circuit: &mut Circuit) {
    circuit.wires.clear();
    circuit.transistors.clear();
    circuit.state.clear();

    let mut parser = Parser {
        circuit,
        pixels: img
            .pixels()
            .map(|color| {
                let mut pixel = Pixel::Void;
                (0..=MAX_CHARGE).for_each(|i| {
                    if *color == CHARGE[i as usize] {
                        pixel = Pixel::Wire(i, None);
                    }
                });
                pixel
            })
            .collect(),
        transistors: Vec::new(),
    };

    let bounds = parser.circuit.bounds;

    /* First pass: wires. */
    for y in 0..bounds.1 {
        for x in 0..bounds.0 {
            if let Pixel::Wire(_, None) = parser.pixels[(y * bounds.0 + x) as usize] {
                let charge = parser.add_wire((x, y));
                parser.circuit.state.push(charge);
            }
        }
    }

    /* Second pass: transistors. */
    while let Some(i) = parser.transistors.pop() {
        parser.add_transistor(i);
    }
}
