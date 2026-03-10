use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainCell {
    Empty,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccupancyCell {
    Empty,
    PowerSource,
    SnakeBody { snakebot_id: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialState {
    pub my_id: i32,
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<TerrainCell>,
    pub my_snakebot_ids: Vec<i32>,
    pub opp_snakebot_ids: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSource {
    pub pos: Cell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnakeBot {
    pub snakebot_id: i32,
    pub body: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnState {
    pub power_sources: Vec<PowerSource>,
    pub snakebots: Vec<SnakeBot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldState {
    pub my_id: i32,
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<TerrainCell>,
    pub occupancy: Vec<OccupancyCell>,
    pub my_snakebot_ids: Vec<i32>,
    pub opp_snakebot_ids: Vec<i32>,
}

impl WorldState {
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn from_initial(initial: InitialState) -> Self {
        let size = initial.width * initial.height;

        Self {
            my_id: initial.my_id,
            width: initial.width as usize,
            height: initial.height as usize,
            terrain: initial.terrain,
            occupancy: vec![OccupancyCell::Empty; size as usize],
            my_snakebot_ids: initial.my_snakebot_ids,
            opp_snakebot_ids: initial.opp_snakebot_ids,
        }
    }

    pub fn apply_turn(&mut self, turn: &TurnState) {
        self.clear_occupancy();

        for power in &turn.power_sources {
            let p = power.pos;
            let idx = self.idx(p.x as usize, p.y as usize);
            self.occupancy[idx] = OccupancyCell::PowerSource;
        }

        for snake in &turn.snakebots {
            for part in &snake.body {
                let idx = self.idx(part.x as usize, part.y as usize);
                self.occupancy[idx] = OccupancyCell::SnakeBody {
                    snakebot_id: snake.snakebot_id,
                };
            }
        }
    }

    pub fn in_bounds(&self, p: Cell) -> bool {
        (p.x as usize) < self.width && (p.y as usize) < self.height
    }

    fn clear_occupancy(&mut self) {
        self.occupancy.fill(OccupancyCell::Empty);
    }

    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.idx(x, y);
                let ch = match self.occupancy[idx] {
                    OccupancyCell::PowerSource => '*',
                    OccupancyCell::SnakeBody { snakebot_id } => {
                        if self.my_snakebot_ids.iter().any(|&id| id == snakebot_id) {
                            'M'
                        } else {
                            'S'
                        }
                    }
                    OccupancyCell::Empty => match self.terrain[idx] {
                        TerrainCell::Wall => '#',
                        TerrainCell::Empty => '.',
                    },
                };
                out.push(ch);
            }
            if y + 1 < self.height {
                out.push('\n');
            }
        }
        out
    }
}

impl fmt::Display for WorldState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_ascii())
    }
}
