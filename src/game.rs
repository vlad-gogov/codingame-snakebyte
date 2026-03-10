use std::fmt;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

pub type Grid<T> = Vec<Vec<T>>;

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
    pub width: i32,
    pub height: i32,
    pub terrain: Grid<TerrainCell>,
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
    pub terrain: Grid<TerrainCell>,
    pub occupancy: Grid<OccupancyCell>,
    pub my_snakebot_ids: HashSet<i32>,
    pub opp_snakebot_ids: HashSet<i32>,
}

impl WorldState {
    pub fn from_initial(initial: &InitialState) -> Option<Self> {
        let width = usize::try_from(initial.width).ok()?;
        let height = usize::try_from(initial.height).ok()?;
        if width == 0 || height == 0 || initial.terrain.len() != height {
            return None;
        }

        for row in &initial.terrain {
            if row.len() != width {
                return None;
            }
        }

        let occupancy = vec![vec![OccupancyCell::Empty; width]; height];
        Some(Self {
            my_id: initial.my_id,
            width,
            height,
            terrain: initial.terrain.clone(),
            occupancy,
            my_snakebot_ids: initial.my_snakebot_ids.iter().copied().collect(),
            opp_snakebot_ids: initial.opp_snakebot_ids.iter().copied().collect(),
        })
    }

    pub fn apply_turn(&mut self, turn: &TurnState) {
        self.clear_occupancy();

        for power in &turn.power_sources {
            if let Some((x, y)) = self.to_index(power.pos) {
                self.occupancy[y][x] = OccupancyCell::PowerSource;
            }
        }

        for snake in &turn.snakebots {
            for part in &snake.body {
                if let Some((x, y)) = self.to_index(*part) {
                    self.occupancy[y][x] = OccupancyCell::SnakeBody {
                        snakebot_id: snake.snakebot_id,
                    };
                }
            }
        }
    }

    pub fn in_bounds(&self, p: Cell) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height
    }

    fn clear_occupancy(&mut self) {
        for row in &mut self.occupancy {
            row.fill(OccupancyCell::Empty);
        }
    }

    fn to_index(&self, p: Cell) -> Option<(usize, usize)> {
        if !self.in_bounds(p) {
            return None;
        }
        Some((usize::try_from(p.x).ok()?, usize::try_from(p.y).ok()?))
    }

    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = match self.occupancy[y][x] {
                    OccupancyCell::PowerSource => '*',
                    OccupancyCell::SnakeBody { snakebot_id } => {
                        if self.my_snakebot_ids.contains(&snakebot_id) {
                            'M'
                        } else if self.opp_snakebot_ids.contains(&snakebot_id) {
                            'E'
                        } else {
                            'S'
                        }
                    }
                    OccupancyCell::Empty => match self.terrain[y][x] {
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
