use crate::game::{Cell, InitialState, PowerSource, SnakeBot, Terrain, TurnState, MAX_CELLS, MAX_SNAKE_LEN};
use std::io::BufRead;
use std::str::FromStr;

fn parse<T: FromStr>(s: &str) -> Result<T, T::Err> {
    s.trim().parse::<T>()
}

fn parse_i32(s: &str) -> Option<i32> {
	parse::<i32>(s).ok()
}

pub struct InputReader<R: BufRead> {
	reader: R,
}

impl<R: BufRead> InputReader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			reader,
		}
	}

	fn read_i32_line(&mut self) -> Option<i32> {
		let s = self.read_line()?;
		parse_i32(&s)
    }

	fn read_line(&mut self) -> Option<String> {
		let mut line = String::new();
		self.reader.read_line(&mut line).ok()?;
		Some(line.trim().to_owned())
	}

	pub fn read_initial_state(&mut self) -> Option<InitialState> {
		let my_id = self.read_i32_line()?;
		let width = self.read_i32_line()? as usize;
		let height = self.read_i32_line()? as usize;

		let mut terrain = [Terrain::Empty; MAX_CELLS];
		for y in 0..height {
			let row = self.read_line()?;
			for (x, ch) in row.chars().enumerate() {
				terrain[y * width + x] = match ch {
					'#' => Terrain::Wall,
					_ => Terrain::Empty,
				};
			}
		}

		let snakebots_per_player = self.read_i32_line()? as usize;
		let mut my_snakebot_ids = Vec::with_capacity(snakebots_per_player);
		for _ in 0..snakebots_per_player {
			my_snakebot_ids.push(self.read_i32_line()?);
		}

		let mut opp_snakebot_ids = Vec::with_capacity(snakebots_per_player);
		for _ in 0..snakebots_per_player {
			opp_snakebot_ids.push(self.read_i32_line()?);
		}

		Some(InitialState {
			my_id,
			width,
			height,
			terrain,
			my_snakebot_ids,
			opp_snakebot_ids,
		})
	}

	pub fn read_turn_state(&mut self, width: usize) -> Option<TurnState> {
		let power_source_count = self.read_i32_line()? as usize;

		let mut power_sources = Vec::with_capacity(power_source_count);
		for _ in 0..power_source_count {
            let line = self.read_line()?;
            let mut it = line.split_whitespace();
			let x: usize = it.next()?.parse().ok()?;
			let y: usize = it.next()?.parse().ok()?;
			let pos = (y * width + x) as Cell;
            power_sources.push(PowerSource { pos });
		}

		let snakebot_count = self.read_i32_line()? as usize;
		let mut snakebots = Vec::with_capacity(snakebot_count);
		for _ in 0..snakebot_count {
			let row = self.read_line()?;
			let mut parts = row.splitn(2, ' ');
			let snakebot_id = parse_i32(parts.next()?)?;
			let body_raw = parts.next().unwrap_or("");
			let mut body = [0 as Cell; MAX_SNAKE_LEN];
			let mut len = 0;
            if !body_raw.is_empty() {
                for token in body_raw.split(':') {
                    let (x, y) = token.split_once(',')?;
					let x: usize = x.parse().ok()?;
        			let y: usize = y.parse().ok()?;
        			body[len] = (y * width + x) as Cell;
					len += 1;
                }
            }
			snakebots.push(SnakeBot { 
				id: snakebot_id,
    			len,
    			body, 
			});
		}

		Some(TurnState {
			power_sources,
			snakebots,
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::game::{Terrain, TurnState};
	use super::InputReader;
	use std::io::Cursor;

	#[test]
	fn parses_initial_state() {
		let data = "1\n4\n3\n....\n.#..\n....\n2\n10\n11\n20\n21\n";
		let cursor = Cursor::new(data.as_bytes());
		let mut reader = InputReader::new(cursor);

		let state = reader.read_initial_state().expect("initial state");
		assert_eq!(state.my_id, 1);
		assert_eq!(state.width, 4);
		assert_eq!(state.height, 3);
		assert_eq!(state.terrain[1 * 4 + 1], Terrain::Wall);
		assert_eq!(state.my_snakebot_ids, vec![10, 11]);
		assert_eq!(state.opp_snakebot_ids, vec![20, 21]);
	}

	#[test]
	fn parses_turn_state() {
		let data = "2\n1 2\n5 6\n3\n10 0,0:0,1:0,2\n20 5,5\n21 1,1:1,2\n";
		let cursor = Cursor::new(data.as_bytes());
		let mut reader = InputReader::new(cursor);

		let width = 10;
		let turn = reader.read_turn_state(width).expect("turn state");
		assert_eq!(turn.power_sources.len(), 2);
		assert_eq!(turn.power_sources[0].pos, (2 * width + 1) as u16);
		assert_eq!(turn.snakebots.len(), 3);
		assert_eq!(turn.snakebots[0].id, 10);
		assert_eq!(turn.snakebots[0].len, 3);
		assert_eq!(turn.snakebots[0].body[0], 0);
	}

	#[test]
	fn reads_multiple_turns_until_eof() {
		let data = "0\n1\n10 0,0\n1\n3 4\n1\n10 0,1\n";
		let cursor = Cursor::new(data.as_bytes());
		let mut reader = InputReader::new(cursor);

		let width = 10;
		let first = reader.read_turn_state(width).expect("first turn");
		let second = reader.read_turn_state(width).expect("second turn");
		let third = reader.read_turn_state(width);

		assert_eq!(first.power_sources.len(), 0);
		assert_eq!(first.snakebots.len(), 1);
		assert_eq!(second.power_sources[0].pos, (4 * width + 3) as u16);
		assert!(third.is_none());
	}

	#[test]
	fn parses_full_snake_body_not_just_first_cell() {
		let data = "0\n1\n10 0,0:0,1:0,2:0,3\n";
		let cursor = Cursor::new(data.as_bytes());
		let mut reader = InputReader::new(cursor);

		let width = 10;
		let turn: TurnState = reader.read_turn_state(width).expect("turn state");
		assert_eq!(turn.snakebots.len(), 1);
		assert_eq!(turn.snakebots[0].len, 4);
		assert_eq!(turn.snakebots[0].body[3], (3 * width + 0) as u16);
	}

	#[test]
	fn accepts_snake_with_empty_body() {
		let data = "0\n1\n10\n";
		let cursor = Cursor::new(data.as_bytes());
		let mut reader = InputReader::new(cursor);

		let width = 10;
		let turn = reader.read_turn_state(width).expect("turn state");
		assert_eq!(turn.snakebots.len(), 1);
	}

	#[test]
	fn rejects_invalid_snake_body_token() {
		let data = "0\n1\n10 0;0\n";
		let cursor = Cursor::new(data.as_bytes());
		let mut reader = InputReader::new(cursor);

		let width = 10;
		assert!(reader.read_turn_state(width).is_none());
	}
}
