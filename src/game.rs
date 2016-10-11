use std::fmt;

#[derive(Clone)]
struct Field {
    alive: bool,
    // tuples pointing in rid; pos = a*size + b
    neighbours: Vec<(usize, usize)>,
}

impl Field {
    fn new(x: usize, y: usize, size: usize) -> Field{
        let mut neighbours = Vec::with_capacity(8);
        
        // vertical neighbours
        // y == 0 -> no upper neighbours
        if y != 0 {
            if x == 0 {
                neighbours.push((x, y-1));
                neighbours.push((x+1, y-1));
            } else if x == size - 1 {
                neighbours.push((x-1, y-1));
                neighbours.push((x, y-1));
            } else {
                neighbours.push((x-1, y-1));
                neighbours.push((x, y-1));
                neighbours.push((x+1, y-1));
            }
        }
        // y == size -1 no lower neighours
        if y != size -1 {
            if x == 0 {
                neighbours.push((x, y+1));
                neighbours.push((x+1, y+1));
            } else if x == size - 1 {
                neighbours.push((x-1, y+1));
                neighbours.push((x, y+1));
            } else {
                neighbours.push((x-1, y+1));
                neighbours.push((x, y+1));
                neighbours.push((x+1, y+1));
            }
        }

        // horisontal neighbours
        if x != 0 {
            neighbours.push((x-1, y));
        }

        if x != size -1 {
            neighbours.push((x+1, y));
        }

        Field {
            alive: false,
            neighbours: neighbours,
        }
    }
}

#[derive(Clone)]
pub struct Grid {
    size: usize,
    grid: Vec<Field>,
    generation: u32,
}

impl Grid {
    pub fn new(size: usize) -> Grid {
        let mut grid = Vec::with_capacity(size*size);

        // generate each field with its neighbours
        for y in 0..size {
            for x in 0..size {
                grid.push(Field::new(x, y, size));
            }
        }

        Grid {
            size: size,
            grid: grid,
            generation: 0,
        }
    }
    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn is_field_alive(&self, x: usize, y:usize) -> bool {
        self.grid[y*self.size+x].alive
    }


    pub fn get_generation(&self) -> u32 {
        self.generation
    }

}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "generation: {}\n", self.generation));
        let vert_line: String = vec!['-';self.size].into_iter().collect();
        try!(write!(f, "+{}+\n", vert_line));
        for y in 0..self.size {
            try!(write!(f, "|"));
            for x in 0..self.size {
                if self.grid[self.size*y+x].alive {
                    try!(write!(f, "#"));
                } else {
                    try!(write!(f, " "));
                }
            }
            try!(write!(f, "|\n"));
        }
        write!(f, "+{}+\n", vert_line)
    }
}

pub struct Game {
    board: Grid,
    born: Vec<u8>,
    survive: Vec<u8>,
}

impl Game {

    /// Create a new game with the default rules 3/23
    pub fn new(size: usize) -> Game {
        Game {
            board: Grid::new(size),
            born: vec![3],
            survive: vec![2,3],
        }
    }

    pub fn new_with_rules(size: usize, born: Vec<u8>, survive: Vec<u8>) -> Game {
        Game {
            board: Grid::new(size),
            born: born,
            survive: survive,
        }
    }

    pub fn resize(&mut self, size: usize) {
        self.board = Grid::new(size);
    }

    pub fn toggle_field(&mut self, x: usize, y: usize) -> bool {
        let pos = y*self.board.size+x;
        self.board.grid[pos].alive = !self.board.grid[pos].alive;
        self.board.grid[pos].alive

    }

    pub fn set_rules(&mut self, born: Vec<u8>, survive: Vec<u8>) {
        self.born = born;
        self.survive = survive;
    }

    pub fn get_board(&self) -> Grid {
        self.board.clone()
    }
}

impl Iterator for Game {
    type Item = Grid;

    fn next(&mut self) -> Option<Grid> {

        let old = self.board.clone();

        let size = self.board.size;
        let mut new: Vec<Field> = Vec::with_capacity(size*size);

        for f in self.board.grid.clone().into_iter() {
            let neighbours = f.neighbours;
            let alive_neighbours = neighbours
                                    .iter()
                                    .fold(0, |sum, f|
                                          if self.board.grid[f.1*size+f.0].alive {
                                            sum + 1
                                          }
                                          else { sum });
            new.push(Field {
                alive: if f.alive {
                    match self.survive.iter().find(|x| **x == alive_neighbours) {
                        Some(_) => true,
                        None    => false,
                    }
                } else {
                    match self.born.iter().find(|x| **x == alive_neighbours) {
                        Some(_) => true,
                        None    => false,
                    }
                },
                neighbours: neighbours,
            });
        }



        self.board = Grid{
            size: size,
            grid: new,
            generation: self.board.generation + 1,
        };

        Some(old)
    }
}
