use crate::*;

/// Returns Von Neumann neighbourhood coordinates as an array [up, down, left, right].
fn neighbourhood(coord: (usize, usize)) -> [(usize, usize); 4] {
    [
        (coord.0 - 1, coord.1),
        (coord.0 + 1, coord.1),
        (coord.0, coord.1 - 1),
        (coord.0, coord.1 + 1),
    ]
}

#[derive(Copy, Clone, PartialEq)]
enum Cell {
    Wire(Option<usize>),
    Transistor,
    Void,
}

struct Parser {
    circuit: Circuit,
    pixels: Vec<Cell>,
}

impl Parser {
    fn get(&self, coord: (usize, usize)) -> Option<&Cell> {
        self.pixels.get(coord.1 * self.circuit.bounds.0 + coord.0)
    }

    fn set(&mut self, coord: (usize, usize), cell: Cell) {
        self.pixels[coord.1 * self.circuit.bounds.0 + coord.0] = cell
    }

    fn count_neighbours(&self, coord: (usize, usize)) -> usize {
        let mut acc = 0;
        for i in neighbourhood(coord).iter() {
            if let Some(pixel) = self.get(*i) {
                match pixel {
                    Cell::Wire(_) => acc += 1,
                    _ => {}
                }
            }
        }

        acc
    }

    fn wire(&mut self, coord: (usize, usize)) {
        let wire_index = self.circuit.wires.len();
        self.circuit.wires.push(Wire {
            is_source: false,
            transistors: Vec::new(),
            pixels: Vec::new(),
        });

        let mut stack = vec![(coord, coord)];
        while let Some((coord, parent)) = stack.pop() {
            match self.get(coord).unwrap_or(&Cell::Void) {
                Cell::Wire(None) => {
                    if self.is_source(coord) {
                        self.circuit.wires[wire_index].is_source = true;
                    }

                    self.circuit.wires[wire_index].pixels.push(coord);

                    self.set(coord, Cell::Wire(Some(wire_index)));

                    for neighbour in neighbourhood(coord).iter() {
                        stack.push((*neighbour, coord));
                    }
                }
                Cell::Void => match self.count_neighbours(coord) {
                    4 => {
                        /* The current coordinate + difference from the previous cell
                         * should take us to the opposite side of the crossing. */
                        let jump = (
                            (coord.0 as isize + (coord.0 as isize - parent.0 as isize)) as usize,
                            (coord.1 as isize + (coord.1 as isize - parent.1 as isize)) as usize,
                        );
                        stack.push((jump, coord));
                    }
                    3 => {
                        /* Possible transistor to be marked. In the end not all of those will become transistors,
                         * as we need to run some checks after the wires are assigned. */
                        self.set(coord, Cell::Transistor)
                    }
                    _ => continue,
                },
                Cell::Wire(Some(_)) | Cell::Transistor => {}
            }
        }
    }

    fn transistor(&mut self, coord: (usize, usize)) {
        let (up, down, left, right) = (0, 1, 2, 3);
        let neighbourhood = neighbourhood(coord);
        let mut neighbours = [Option::<usize>::None; 4];

        for (i, coord) in neighbourhood.iter().enumerate() {
            if let Some(Cell::Wire(Some(wire))) = self.get(*coord) {
                neighbours[i] = Some(*wire);
            }
        }

        let mut transistor = Transistor {
            pins: [0; 2],
            base: 0,
            position: coord,
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

        if transistor.pins[0] == transistor.base || transistor.pins[1] == transistor.base {
            return;
        }

        self.circuit.wires[transistor.pins[0]]
            .transistors
            .push(self.circuit.transistors.len());
        self.circuit.wires[transistor.pins[1]]
            .transistors
            .push(self.circuit.transistors.len());
        self.circuit.transistors.push(transistor);
    }

    fn is_source(&self, coord: (usize, usize)) -> bool {
        /* If cells to the right, down and down-right are occupied by wires,
         * we've got ourselves a square which is a power source. */
        for i in [
            (coord.0 + 1, coord.1),     // right
            (coord.0, coord.1 + 1),     // down
            (coord.0 + 1, coord.1 + 1), // right and down
        ]
        .iter()
        {
            if let Some(pixel) = self.get(*i) {
                match pixel {
                    Cell::Void | Cell::Transistor => return false,
                    Cell::Wire(_) => continue,
                }
            }
        }

        true
    }
}

pub fn parse(circuit: &[bool], bounds: (usize, usize)) -> Circuit {
    let mut parser = Parser {
        circuit: Circuit {
            bounds,
            wires: Vec::new(),
            transistors: Vec::new(),
            state: Vec::new(),
        },
        pixels: circuit
            .iter()
            .map(|i| if *i { Cell::Wire(None) } else { Cell::Void })
            .collect(),
    };

    /* First pass: wires. */
    for y in 0..bounds.1 {
        for x in 0..bounds.0 {
            if let Cell::Wire(None) = parser.pixels[(y * bounds.0 + x) as usize] {
                parser.wire((x, y));
            }
        }
    }

    parser.circuit.state.resize(parser.circuit.wires.len(), 0);

    /* Second pass: transistors. */
    for y in 0..bounds.1 {
        for x in 0..bounds.0 {
            if let Cell::Transistor = parser.pixels[(y * bounds.0 + x) as usize] {
                parser.transistor((x, y));
            }
        }
    }

    parser.circuit
}
