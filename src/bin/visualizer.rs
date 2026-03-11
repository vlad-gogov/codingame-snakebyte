use codingame_snakebyte::game::{Terrain, TurnState, WorldState, OCC_EMPTY, OCC_POWER};
use codingame_snakebyte::input_reader::InputReader;
use macroquad::prelude::*;
use std::io;

const MARGIN: f32 = 20.0;
const HUD_HEIGHT: f32 = 72.0;
const AUTO_STEP_SECONDS: f32 = 0.20;

fn window_conf() -> Conf {
    Conf {
        window_title: "Snakebyte Replay Visualizer".to_string(),
        window_width: 1280,
        window_height: 768,
        ..Default::default()
    }
}

fn load_replay() -> Option<(WorldState, Vec<TurnState>)> {
    let stdin = io::stdin();
    let mut input = InputReader::new(stdin.lock());

    let initial = input.read_initial_state()?;
    let base_world = WorldState::from_initial(initial);

    let mut turns = Vec::new();
    while let Some(turn) = input.read_turn_state(base_world.width) {
        turns.push(turn);
    }

    Some((base_world, turns))
}

fn rebuild_world(base: &WorldState, turns: &[TurnState], idx: usize) -> WorldState {
    let mut world = base.clone();

    for turn in turns.iter().take(idx.saturating_add(1)) {
        world.apply_turn(turn);
    }

    world
}

fn draw_world(world: &WorldState, turn_idx: usize, total_turns: usize, paused: bool) {
    clear_background(Color::from_rgba(10, 14, 22, 255));

    let free_w = screen_width() - MARGIN * 2.0;
    let free_h = screen_height() - MARGIN * 2.0 - HUD_HEIGHT;
    let cell_size = (free_w / world.width as f32)
        .min(free_h / world.height as f32)
        .max(2.0);

    let world_w = world.width as f32 * cell_size;
    let offset_x = (screen_width() - world_w) * 0.5;
    let offset_y = MARGIN + HUD_HEIGHT;

    let hud = format!(
        "Turn: {}/{}  |  Paused: {}  |  Controls: Space play/pause, Left/Right step, Home/End jump",
        turn_idx.saturating_add(1),
        total_turns.max(1),
        if paused { "yes" } else { "no" }
    );
    draw_text(&hud, MARGIN, MARGIN + 28.0, 24.0, WHITE);

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            let px = offset_x + x as f32 * cell_size;
            let py = offset_y + y as f32 * cell_size;

            let occ = world.occupancy[idx];
            let color = if occ == OCC_POWER {
                Color::from_rgba(242, 189, 50, 255)
            } else if occ >= 0 {
                let snakebot_id = occ as i32;
                let mut is_my_snake = false;
                for i in 0..world.my_snakebot_count {
                    if world.my_snakebot_ids[i as usize] == snakebot_id as u8 {
                        is_my_snake = true;
                        break;
                    }
                }
                if is_my_snake {
                    Color::from_rgba(255, 105, 180, 255)
                } else {
                    Color::from_rgba(70, 214, 120, 255)
                }
            } else if occ == OCC_EMPTY {
                match world.terrain[idx] {
                    Terrain::Wall => Color::from_rgba(77, 84, 102, 255),
                    Terrain::Empty => Color::from_rgba(18, 24, 35, 255),
                }
            } else {
                Color::from_rgba(18, 24, 35, 255)
            };

            draw_rectangle(px, py, cell_size, cell_size, color);
            draw_rectangle_lines(
                px,
                py,
                cell_size,
                cell_size,
                1.0,
                Color::from_rgba(32, 40, 56, 255),
            );
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let Some((base_world, turns)) = load_replay() else {
        loop {
            clear_background(BLACK);
            draw_text(
                "No replay data on stdin. Usage: cargo run --bin visualizer < replay.txt",
                20.0,
                40.0,
                28.0,
                RED,
            );
            next_frame().await;
        }
    };

    let mut turn_idx = 0usize;
    let mut paused = true;
    let mut accumulator = 0.0f32;
    let mut dirty = true;
    let mut world = base_world.clone();

    if !turns.is_empty() {
        world.apply_turn(&turns[0]);
        dirty = false;
    }

    loop {
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }

        if is_key_pressed(KeyCode::Right) && !turns.is_empty() {
            turn_idx = (turn_idx + 1).min(turns.len() - 1);
            dirty = true;
        }

        if is_key_pressed(KeyCode::Left) && !turns.is_empty() {
            turn_idx = turn_idx.saturating_sub(1);
            dirty = true;
        }

        if is_key_pressed(KeyCode::Home) {
            turn_idx = 0;
            dirty = true;
        }

        if is_key_pressed(KeyCode::End) && !turns.is_empty() {
            turn_idx = turns.len() - 1;
            dirty = true;
        }

        if !paused && !turns.is_empty() {
            accumulator += get_frame_time();
            if accumulator >= AUTO_STEP_SECONDS {
                accumulator = 0.0;
                if turn_idx + 1 < turns.len() {
                    turn_idx += 1;
                    dirty = true;
                } else {
                    paused = true;
                }
            }
        }

        if dirty {
            world = rebuild_world(&base_world, &turns, turn_idx);
            dirty = false;
        }

        draw_world(&world, turn_idx, turns.len(), paused);
        next_frame().await;
    }
}
