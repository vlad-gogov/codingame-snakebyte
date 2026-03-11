use std::fmt;

pub const MAX_WIDTH: usize = 44;
pub const MAX_HEIGHT: usize = 24;
pub const MAX_CELLS: usize = MAX_WIDTH * MAX_HEIGHT;

pub const MAX_SNAKES: usize = 16;
pub const MAX_SNAKE_LEN: usize = 128;
pub const MAX_FOOD: usize = 128;

pub type Cell = u16;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Empty = 0,
    Wall = 1,
}

#[derive(Clone)]
pub struct SnakeBot {
    pub id: i32,
    pub len: usize,
    pub body: [Cell; MAX_SNAKE_LEN],
}

#[derive(Clone)]
pub struct InitialState {
    pub my_id: i32,
    pub width: usize,
    pub height: usize,
    pub terrain: [Terrain; MAX_CELLS],
    pub my_snakebot_ids: Vec<i32>,
    pub opp_snakebot_ids: Vec<i32>,
}

#[derive(Clone)]
pub struct PowerSource {
    pub pos: Cell,
}

#[derive(Clone)]
pub struct TurnState {
    pub power_sources: Vec<PowerSource>,
    pub snakebots: Vec<SnakeBot>,
}

pub const OCC_EMPTY: i16 = -1;
pub const OCC_POWER: i16 = -2;

#[derive(Clone)]
pub struct WorldState {
    pub my_id: i32,
    pub width: usize,
    pub height: usize,
    pub terrain: [Terrain; MAX_CELLS],

    /// -1 empty
    /// -2 power
    /// >=0 snake id
    pub occupancy: [i16; MAX_CELLS],
    pub my_snakebot_ids: [u8; MAX_SNAKES],
    pub my_snakebot_count: u8,
    pub opp_snakebot_ids: [u8; MAX_SNAKES],
    pub opp_snakebot_count: u8,
}

impl WorldState {

    #[inline(always)]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline(always)]
    fn xy(&self, cell: Cell) -> (usize, usize) {
        let c = cell as usize;
        (c % self.width, c / self.width)
    }

    pub fn from_initial(initial: InitialState) -> Self {
        let mut my_ids = [0u8; MAX_SNAKES];
        for (i, &id) in initial.my_snakebot_ids.iter().enumerate().take(MAX_SNAKES) {
            my_ids[i] = id as u8;
        }

        let mut opp_ids = [0u8; MAX_SNAKES];
        for (i, &id) in initial.opp_snakebot_ids.iter().enumerate().take(MAX_SNAKES) {
            opp_ids[i] = id as u8;
        }
        Self {
            my_id: initial.my_id,
            width: initial.width,
            height: initial.height,
            terrain: initial.terrain,
            occupancy: [-1; MAX_CELLS],
            my_snakebot_ids: my_ids,
            my_snakebot_count: initial.my_snakebot_ids.len() as u8,
            opp_snakebot_ids: opp_ids,
            opp_snakebot_count: initial.opp_snakebot_ids.len() as u8,
        }
    }

    #[inline]
    fn clear_occupancy(&mut self) {
        self.occupancy.fill(OCC_EMPTY);
    }

    pub fn apply_turn(&mut self, turn: &TurnState) {

        self.clear_occupancy();

        // power sources
        for power in &turn.power_sources {

            let idx = power.pos as usize;

            self.occupancy[idx] = OCC_POWER;
        }

        // snakes
        for snake in &turn.snakebots {

            for i in 0..snake.len {

                let cell = snake.body[i] as usize;

                self.occupancy[cell] = snake.id as i16;
            }
        }
    }

    pub fn render_ascii(&self) -> String {

        let mut out = String::with_capacity(self.width * self.height + self.height);

        for y in 0..self.height {

            for x in 0..self.width {

                let idx = self.idx(x, y);

                let ch = match self.occupancy[idx] {

                    OCC_POWER => '*',

                    id if id >= 0 => {

                        if self.my_snakebot_ids.iter().any(|&s| s == id as u8) {
                            'M'
                        } else {
                            'S'
                        }
                    }

                    _ => match self.terrain[idx] {

                        Terrain::Wall => '#',

                        Terrain::Empty => '.',
                    }
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

pub const DIR_UP: usize = 0;
pub const DIR_RIGHT: usize = 1;
pub const DIR_DOWN: usize = 2;
pub const DIR_LEFT: usize = 3;

pub const DIR_COUNT: usize = 4;

fn build_neighbors(width: usize, height: usize) -> [[Cell; DIR_COUNT]; MAX_CELLS] {

    let mut neighbors = [[0 as Cell; DIR_COUNT]; MAX_CELLS];

    for y in 0..height {
        for x in 0..width {

            let idx = (y * width + x) as Cell;

            // UP
            neighbors[idx as usize][DIR_UP] =
                if y > 0 { ((y - 1) * width + x) as Cell } else { idx };

            // RIGHT
            neighbors[idx as usize][DIR_RIGHT] =
                if x + 1 < width { (y * width + (x + 1)) as Cell } else { idx };

            // DOWN
            neighbors[idx as usize][DIR_DOWN] =
                if y + 1 < height { ((y + 1) * width + x) as Cell } else { idx };

            // LEFT
            neighbors[idx as usize][DIR_LEFT] =
                if x > 0 { (y * width + (x - 1)) as Cell } else { idx };
        }
    }

    neighbors
}

pub struct FastWorld {

    pub width: usize,
    pub height: usize,
    pub size: usize,

    pub terrain: [Terrain; MAX_CELLS],

    /// -1 empty
    /// -2 power
    /// >=0 snake id
    pub occupancy: [i16; MAX_CELLS],

    pub my_snakebot_ids: [u8; MAX_SNAKES],
    pub my_snakebot_count: u8,
    pub opp_snakebot_ids: [u8; MAX_SNAKES],
    pub opp_snakebot_count: u8,

    pub snakes: [SnakeBot; MAX_SNAKES],
    pub snake_count: usize,

    pub power: [Cell; MAX_FOOD],
    pub power_count: usize,

    pub neighbors: [[Cell; DIR_COUNT]; MAX_CELLS],
}

impl FastWorld {

    pub fn from_world(world: &WorldState, turn: &TurnState) -> Self {
        let mut fast = Self {

            width: world.width,
            height: world.height,
            size: world.width * world.height,

            terrain: world.terrain,

            occupancy: [OCC_EMPTY; MAX_CELLS],

            my_snakebot_ids: world.my_snakebot_ids,
            my_snakebot_count: 0,
            opp_snakebot_ids: world.opp_snakebot_ids,
            opp_snakebot_count: 0,
            snakes: unsafe { std::mem::zeroed() },
            snake_count: 0,

            power: [0; MAX_FOOD],
            power_count: 0,

            neighbors: build_neighbors(world.width, world.height),
        };

        // power sources
        for (i, p) in turn.power_sources.iter().enumerate() {

            fast.power[i] = p.pos;

            fast.occupancy[p.pos as usize] = OCC_POWER;
        }

        fast.power_count = turn.power_sources.len();

        // snakes
        for (i, s) in turn.snakebots.iter().enumerate() {

            fast.snakes[i] = s.clone();

            for j in 0..s.len {

                let c = s.body[j] as usize;

                fast.occupancy[c] = s.id as i16;
            }
            let mut is_my_snake = false;
            for i in 0..world.my_snakebot_count {
                if world.my_snakebot_ids[i as usize] == s.id as u8 {
                    is_my_snake = true;
                    break;
                }
            }
            if is_my_snake {
                let count = fast.my_snakebot_count as usize;
                if count < MAX_SNAKES {
                    fast.my_snakebot_ids[count] = s.id as u8;
                    fast.my_snakebot_count += 1;
                }
            } else {
                let count = fast.opp_snakebot_count as usize;
                if count < MAX_SNAKES {
                    fast.opp_snakebot_ids[count] = s.id as u8;
                    fast.opp_snakebot_count += 1;
                }
            }
        }

        fast.snake_count = turn.snakebots.len();

        fast
    }

    #[inline]
    pub fn clear_occupancy(&mut self) {
        self.occupancy.fill(OCC_EMPTY);
    }

    // #[inline(always)]
    // pub fn can_move(&self, from: Cell, dir: usize) -> bool {
    //     let to = self.neighbors[from as usize][dir];
    //     if to == from {
    //         return false;
    //     }
    //     let occ = self.occupancy[to as usize];
    //     occ == OCC_EMPTY || occ == OCC_POWER
    // }

    // pub fn move_snake(&mut self, sid: usize, dir: usize) {
    //     let snake = &mut self.snakes[sid];
    //     let head = snake.body[0] as i32;
    //     let new_head = self.neighbors[head as usize][dir];

    //     // shift body
    //     for i in (1..snake.len).rev() {
    //         snake.body[i] = snake.body[i-1];
    //     }

    //     snake.body[0] = new_head;
    // }

    #[inline(always)]
    pub fn is_free(&self, cell: Cell) -> bool {
        let v = self.occupancy[cell as usize];
        v == OCC_EMPTY || v == OCC_POWER
    }

    #[inline(always)]
    pub fn is_wall(&self, cell: Cell) -> bool {
        self.terrain[cell as usize] == Terrain::Wall
    }

    #[inline(always)]
    pub fn is_snake(&self, cell: Cell) -> bool {
        self.occupancy[cell as usize] >= 0
    }

    pub fn bfs(&self, start: Cell, dist: &mut [i16; MAX_CELLS]) {
        let mut queue = [0 as Cell; MAX_CELLS];
        let mut head = 0;
        let mut tail = 0;

        queue[tail] = start;
        tail += 1;

        dist[start as usize] = 0;

        while head < tail {
            let cur = queue[head];
            head += 1;
            let d = dist[cur as usize] + 1;
            for dir in 0..DIR_COUNT {
                let next = self.neighbors[cur as usize][dir];
                if next == cur {
                    continue;
                }
                if dist[next as usize] != -1 {
                    continue;
                }
                if self.occupancy[next as usize] >= 0 {
                    continue;
                }
                dist[next as usize] = d;
                queue[tail] = next;
                tail += 1;
            }
        }
    }

    pub fn simulate_turn(&mut self, moves: &[u8; MAX_SNAKES]) {
        let mut new_heads = [0 as Cell; MAX_SNAKES];

        // New heads
        for i in 0..self.snake_count {
            let snake = &self.snakes[i];
            if snake.len == 0 {
                continue;
            }
            let head = snake.body[0];
            let dir = moves[i] as usize;
            let next = self.neighbors[head as usize][dir];
            new_heads[i] = next;
        }

        // Check for collisions and walls
        let mut dead = [false; MAX_SNAKES];
        for i in 0..self.snake_count {
            let pos = new_heads[i];
            let occ = self.occupancy[pos as usize];
            if occ >= 0 || occ == OCC_POWER {
                dead[i] = true;
            }
        }

        // Head to head collisions
        let mut head_count = [0u8; MAX_CELLS];
        for i in 0..self.snake_count {
            if self.snakes[i].len == 0 || dead[i] { 
                continue; 
            }
            head_count[new_heads[i] as usize] += 1;
        }

        for i in 0..self.snake_count {
            if self.snakes[i].len == 0 || dead[i] { 
                continue;
            }
            if head_count[new_heads[i] as usize] > 1 {
                dead[i] = true;
            }
        }

        // Delete tail
        for i in 0..self.snake_count {
            let snake = &self.snakes[i];
            if snake.len == 0 {
                continue;
            }
            let tail = snake.body[snake.len - 1];
            self.occupancy[tail as usize] = OCC_EMPTY;
        }

        // Move snake
        for i in 0..self.snake_count {
            if dead[i] {
                self.snakes[i].len = 0;
                continue;
            }
            let snake = &mut self.snakes[i];
            let new_head = new_heads[i];
            for j in (1..snake.len).rev() {
                snake.body[j] = snake.body[j - 1];
            }
            snake.body[0] = new_head;
            self.occupancy[new_head as usize] = snake.id as i16;
        }

        // Power up snakes
        for i in 0..self.snake_count {
            let snake = &mut self.snakes[i];
            if snake.len == 0 {
                continue;
            }
            let head = snake.body[0];
            if self.occupancy[head as usize] == OCC_POWER {
                snake.body[snake.len] = snake.body[snake.len - 1];
                snake.len += 1;
                self.occupancy[head as usize] = snake.id as i16;
            }
        }
    }

    pub fn choose_move(&self, idx: usize) -> u8 {
        let head = self.snakes[idx].body[0];

        let mut best_dir = 0;
        let mut best_score = -1;

        for dir in 0..DIR_COUNT {
            let next = self.neighbors[head as usize][dir];

            if !self.is_free(next) { 
                continue;
            }

            if self.occupancy[next as usize] == OCC_POWER {
                return dir as u8;
            }

            let mut dist = [-1i16; MAX_CELLS];
            self.bfs(next, &mut dist);

            let mut area = 0;
            for d in dist.iter() {
                if *d >= 0 { 
                    area += 1;
                }
            }

            if area > best_score {
                best_score = area;
                best_dir = dir;
            }
        }

        best_dir as u8
    }

    pub fn moves_to_text(&self) -> String {
        let mut out = String::new();

        for i in 0..self.my_snakebot_count {
            let id = self.my_snakebot_ids[i as usize] as usize;
            let snake = &self.snakes[id];
            if snake.len == 0 { 
                continue; 
            }

            let dir = self.choose_move(id);

            let dir_text = match dir as usize {
                DIR_UP => "UP",
                DIR_RIGHT => "RIGHT",
                DIR_DOWN => "DOWN",
                DIR_LEFT => "LEFT",
                _ => "WAIT",
            };

            if !out.is_empty() {
                out.push(';');
            }

            out.push_str(&format!("{} {}", snake.id, dir_text));
        }

        out
    }

}

