use iced::keyboard;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Subscription, Task, Theme};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum Message {
    Move(Direction),
    NewGame,
    ToggleDarkMode,
    Quit,
    Undo,
    KeyPressed(keyboard::Key, keyboard::Modifiers),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    board: [[u32; 4]; 4],
    score: u32,
    game_over: bool,
    won: bool,
    dark_mode: bool,
    history: Vec<GameState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GameState {
    board: [[u32; 4]; 4],
    score: u32,
    game_over: bool,
    won: bool,
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            board: [[0; 4]; 4],
            score: 0,
            game_over: false,
            won: false,
            dark_mode: false,
            history: Vec::new(),
        };
        game.add_random_tile();
        game.add_random_tile();
        game
    }

    fn new_with_theme(dark_mode: bool) -> Self {
        let mut game = Game {
            board: [[0; 4]; 4],
            score: 0,
            game_over: false,
            won: false,
            dark_mode,
            history: Vec::new(),
        };
        game.add_random_tile();
        game.add_random_tile();
        game
    }

    fn add_random_tile(&mut self) {
        let mut empty_cells = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == 0 {
                    empty_cells.push((i, j));
                }
            }
        }

        if !empty_cells.is_empty() {
            let mut rng = rand::rng();
            let (i, j) = empty_cells[rng.random_range(0..empty_cells.len())];
            self.board[i][j] = if rng.random_bool(0.9) { 2 } else { 4 };
        }
    }

    fn move_tiles(&mut self, direction: Direction) -> bool {
        // Save current state before making a move
        self.save_state();

        let old_board = self.board;

        match direction {
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::Up => self.move_up(),
            Direction::Down => self.move_down(),
        }

        let moved = old_board != self.board;
        if moved {
            self.add_random_tile();
            self.check_game_state();
        } else {
            // If no move was made, remove the saved state
            self.history.pop();
        }

        moved
    }

    fn move_left(&mut self) {
        for row in 0..4 {
            let mut line = [0; 4];
            let mut pos = 0;

            // Move all tiles to the left
            for col in 0..4 {
                if self.board[row][col] != 0 {
                    line[pos] = self.board[row][col];
                    pos += 1;
                }
            }

            // Merge adjacent tiles
            for i in 0..3 {
                if line[i] != 0 && line[i] == line[i + 1] {
                    line[i] *= 2;
                    self.score += line[i];
                    if line[i] == 2048 && !self.won {
                        self.won = true;
                    }
                    line[i + 1] = 0;
                }
            }

            // Move tiles left again after merging
            let mut merged_line = [0; 4];
            pos = 0;
            for &item in &line {
                if item != 0 {
                    merged_line[pos] = item;
                    pos += 1;
                }
            }

            self.board[row] = merged_line;
        }
    }

    fn move_right(&mut self) {
        for row in 0..4 {
            let mut line = [0; 4];
            let mut pos = 3;

            // Move all tiles to the right
            for col in (0..4).rev() {
                if self.board[row][col] != 0 {
                    line[pos] = self.board[row][col];
                    pos = pos.saturating_sub(1);
                }
            }

            // Merge adjacent tiles
            for i in (1..4).rev() {
                if line[i] != 0 && line[i] == line[i - 1] {
                    line[i] *= 2;
                    self.score += line[i];
                    if line[i] == 2048 && !self.won {
                        self.won = true;
                    }
                    line[i - 1] = 0;
                }
            }

            // Move tiles right again after merging
            let mut merged_line = [0; 4];
            pos = 3;
            for i in (0..4).rev() {
                if line[i] != 0 {
                    merged_line[pos] = line[i];
                    pos = pos.saturating_sub(1);
                }
            }

            self.board[row] = merged_line;
        }
    }

    fn move_up(&mut self) {
        for col in 0..4 {
            let mut line = [0; 4];
            let mut pos = 0;

            // Move all tiles up
            for row in 0..4 {
                if self.board[row][col] != 0 {
                    line[pos] = self.board[row][col];
                    pos += 1;
                }
            }

            // Merge adjacent tiles
            for i in 0..3 {
                if line[i] != 0 && line[i] == line[i + 1] {
                    line[i] *= 2;
                    self.score += line[i];
                    if line[i] == 2048 && !self.won {
                        self.won = true;
                    }
                    line[i + 1] = 0;
                }
            }

            // Move tiles up again after merging
            let mut merged_line = [0; 4];
            pos = 0;
            for &item in &line {
                if item != 0 {
                    merged_line[pos] = item;
                    pos += 1;
                }
            }

            for (row, &value) in merged_line.iter().enumerate() {
                self.board[row][col] = value;
            }
        }
    }

    fn move_down(&mut self) {
        for col in 0..4 {
            let mut line = [0; 4];
            let mut pos = 3;

            // Move all tiles down
            for row in (0..4).rev() {
                if self.board[row][col] != 0 {
                    line[pos] = self.board[row][col];
                    pos = pos.saturating_sub(1);
                }
            }

            // Merge adjacent tiles
            for i in (1..4).rev() {
                if line[i] != 0 && line[i] == line[i - 1] {
                    line[i] *= 2;
                    self.score += line[i];
                    if line[i] == 2048 && !self.won {
                        self.won = true;
                    }
                    line[i - 1] = 0;
                }
            }

            // Move tiles down again after merging
            let mut merged_line = [0; 4];
            pos = 3;
            for i in (0..4).rev() {
                if line[i] != 0 {
                    merged_line[pos] = line[i];
                    pos = pos.saturating_sub(1);
                }
            }

            for (row, &value) in merged_line.iter().enumerate() {
                self.board[row][col] = value;
            }
        }
    }

    fn check_game_state(&mut self) {
        // Check if there are any empty cells
        for row in 0..4 {
            for col in 0..4 {
                if self.board[row][col] == 0 {
                    return; // Game continues
                }
            }
        }

        // Check if any adjacent tiles can be merged
        for row in 0..4 {
            for col in 0..4 {
                let current = self.board[row][col];
                if (row < 3 && self.board[row + 1][col] == current)
                    || (col < 3 && self.board[row][col + 1] == current)
                {
                    return; // Game continues
                }
            }
        }

        self.game_over = true;
    }

    fn save_state(&mut self) {
        let state = GameState {
            board: self.board,
            score: self.score,
            game_over: self.game_over,
            won: self.won,
        };
        self.history.push(state);
    }

    fn undo(&mut self) -> bool {
        if let Some(previous_state) = self.history.pop() {
            self.board = previous_state.board;
            self.score = previous_state.score;
            self.game_over = previous_state.game_over;
            self.won = previous_state.won;
            true
        } else {
            false
        }
    }

    fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    fn reset_tiles(&mut self) {
        // Reset board and game state but preserve history and settings
        self.board = [[0; 4]; 4];
        self.score = 0;
        self.game_over = false;
        self.won = false;
        // Keep history and dark_mode unchanged
        self.add_random_tile();
        self.add_random_tile();
    }

    fn save_file_path() -> Option<PathBuf> {
        dirs::data_dir().and_then(|mut path| {
            path.push("iced-2048");
            std::fs::create_dir_all(&path).ok()?;
            path.push("save.json");
            Some(path)
        })
    }

    fn save(&self) {
        if let Some(path) = Self::save_file_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, json);
            }
        }
    }

    fn load() -> Option<Self> {
        if let Some(path) = Self::save_file_path() {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(game) = serde_json::from_str::<Game>(&contents) {
                    return Some(game);
                }
            }
        }
        None
    }
}

fn update(game: &mut Game, message: Message) -> Task<Message> {
    match message {
        Message::Move(direction) => {
            if !game.game_over {
                game.move_tiles(direction);
                game.save();
            }
        }
        Message::NewGame => {
            let current_dark_mode = game.dark_mode;
            *game = Game::new_with_theme(current_dark_mode);
            game.save();
        }
        Message::ToggleDarkMode => {
            game.dark_mode = !game.dark_mode;
            game.save();
        }
        Message::Quit => {
            return iced::exit();
        }
        Message::Undo => {
            game.undo();
            game.save();
        }
        Message::KeyPressed(key, _modifiers) => {
            if let keyboard::Key::Named(named_key) = key {
                match named_key {
                    keyboard::key::Named::ArrowUp => {
                        if !game.game_over {
                            game.move_tiles(Direction::Up);
                            game.save();
                        }
                    }
                    keyboard::key::Named::ArrowDown => {
                        if !game.game_over {
                            game.move_tiles(Direction::Down);
                            game.save();
                        }
                    }
                    keyboard::key::Named::ArrowLeft => {
                        if !game.game_over {
                            game.move_tiles(Direction::Left);
                            game.save();
                        }
                    }
                    keyboard::key::Named::ArrowRight => {
                        if !game.game_over {
                            game.move_tiles(Direction::Right);
                            game.save();
                        }
                    }
                    keyboard::key::Named::Space => {
                        game.reset_tiles();
                        game.save();
                    }
                    _ => {}
                }
            }
        }
    }
    Task::none()
}

fn view(game: &Game) -> Element<Message> {
    let title = text("2048").size(50);

    let score_text = text(format!("Score: {}", game.score)).size(20);

    let mut board_rows = Vec::new();
    for board_row in 0..4 {
        let mut board_cols = Vec::new();
        for board_col in 0..4 {
            let tile_value = game.board[board_row][board_col];
            let tile_text = if tile_value == 0 {
                String::new()
            } else {
                tile_value.to_string()
            };

            let (background_color, text_color) = if game.dark_mode {
                // Dark mode colors
                let bg = match tile_value {
                    0 => iced::Color::from_rgb(0.2, 0.2, 0.2),
                    2 => iced::Color::from_rgb(0.3, 0.3, 0.3),
                    4 => iced::Color::from_rgb(0.4, 0.4, 0.35),
                    8 => iced::Color::from_rgb(0.5, 0.4, 0.2),
                    16 => iced::Color::from_rgb(0.6, 0.3, 0.2),
                    32 => iced::Color::from_rgb(0.7, 0.2, 0.2),
                    64 => iced::Color::from_rgb(0.8, 0.1, 0.1),
                    128 => iced::Color::from_rgb(0.7, 0.6, 0.1),
                    256 => iced::Color::from_rgb(0.8, 0.7, 0.1),
                    512 => iced::Color::from_rgb(0.9, 0.8, 0.1),
                    1024 => iced::Color::from_rgb(1.0, 0.9, 0.1),
                    2048 => iced::Color::from_rgb(1.0, 0.8, 0.0),
                    4096 => iced::Color::from_rgb(0.9, 0.1, 0.9),
                    8192 => iced::Color::from_rgb(0.8, 0.2, 0.9),
                    16384 => iced::Color::from_rgb(0.7, 0.3, 0.9),
                    32768 => iced::Color::from_rgb(0.6, 0.4, 0.9),
                    65536 => iced::Color::from_rgb(0.5, 0.5, 0.9),
                    131072 => iced::Color::from_rgb(0.4, 0.6, 0.9),
                    262144 => iced::Color::from_rgb(0.3, 0.7, 0.9),
                    524288 => iced::Color::from_rgb(0.2, 0.8, 0.9),
                    1048576 => iced::Color::from_rgb(0.1, 0.9, 0.9),
                    2097152 => iced::Color::from_rgb(0.0, 1.0, 0.9),
                    4194304 => iced::Color::from_rgb(0.0, 0.9, 1.0),
                    _ => iced::Color::from_rgb(0.3, 0.6, 0.9),
                };
                let text_color = if tile_value <= 4 && tile_value > 0 {
                    iced::Color::from_rgb(0.8, 0.8, 0.8)
                } else {
                    iced::Color::WHITE
                };
                (bg, text_color)
            } else {
                // Light mode colors (original)
                let bg = match tile_value {
                    0 => iced::Color::from_rgb(0.8, 0.8, 0.8),
                    2 => iced::Color::from_rgb(0.9, 0.9, 0.85),
                    4 => iced::Color::from_rgb(0.9, 0.85, 0.8),
                    8 => iced::Color::from_rgb(0.9, 0.7, 0.5),
                    16 => iced::Color::from_rgb(0.9, 0.6, 0.4),
                    32 => iced::Color::from_rgb(0.9, 0.5, 0.3),
                    64 => iced::Color::from_rgb(0.9, 0.4, 0.2),
                    128 => iced::Color::from_rgb(0.9, 0.8, 0.4),
                    256 => iced::Color::from_rgb(0.9, 0.8, 0.3),
                    512 => iced::Color::from_rgb(0.9, 0.8, 0.2),
                    1024 => iced::Color::from_rgb(0.9, 0.8, 0.1),
                    2048 => iced::Color::from_rgb(1.0, 0.8, 0.0),
                    4096 => iced::Color::from_rgb(0.8, 0.1, 0.8),
                    8192 => iced::Color::from_rgb(0.7, 0.2, 0.8),
                    16384 => iced::Color::from_rgb(0.6, 0.3, 0.8),
                    32768 => iced::Color::from_rgb(0.5, 0.4, 0.8),
                    65536 => iced::Color::from_rgb(0.4, 0.5, 0.8),
                    131072 => iced::Color::from_rgb(0.3, 0.6, 0.8),
                    262144 => iced::Color::from_rgb(0.2, 0.7, 0.8),
                    524288 => iced::Color::from_rgb(0.1, 0.8, 0.8),
                    1048576 => iced::Color::from_rgb(0.0, 0.9, 0.8),
                    2097152 => iced::Color::from_rgb(0.0, 0.8, 0.9),
                    4194304 => iced::Color::from_rgb(0.0, 0.7, 0.9),
                    _ => iced::Color::from_rgb(0.2, 0.2, 0.2),
                };
                let text_color = if tile_value <= 4 && tile_value > 0 {
                    iced::Color::from_rgb(0.4, 0.4, 0.4)
                } else {
                    iced::Color::WHITE
                };
                (bg, text_color)
            };

            let tile = container(text(tile_text).size(28).color(text_color))
                .width(90)
                .height(90)
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(background_color)),
                    border: iced::Border {
                        color: if game.dark_mode {
                            iced::Color::from_rgb(0.4, 0.4, 0.4)
                        } else {
                            iced::Color::from_rgb(0.7, 0.7, 0.7)
                        },
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                });

            board_cols.push(tile.into());
        }
        board_rows.push(row(board_cols).spacing(8).into());
    }

    let board = column(board_rows).spacing(8);

    let new_game_button = button("New Game").on_press(Message::NewGame).padding(12);

    let toggle_theme_button = button(if game.dark_mode {
        "Light Mode"
    } else {
        "Dark Mode"
    })
    .on_press(Message::ToggleDarkMode)
    .padding(12);

    let undo_button = if game.can_undo() {
        button("Undo").on_press(Message::Undo).padding(12)
    } else {
        button("Undo").padding(12) // Disabled button
    };

    let status_text = if game.won && !game.game_over {
        text("You Win! Keep playing or start a new game.").size(18)
    } else if game.game_over {
        text("Game Over! Try again.").size(18)
    } else {
        text("Use WASD/arrows to move • Space to reset • Z to undo • P for dark mode • Ctrl+Q to quit")
            .size(16)
    };

    let button_row = row![new_game_button, toggle_theme_button, undo_button].spacing(12);

    let content = column![title, score_text, board, status_text, button_row]
        .spacing(20)
        .align_x(iced::Alignment::Center);

    container(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center(iced::Length::Fill)
        .padding(20)
        .into()
}

fn subscription(_game: &Game) -> Subscription<Message> {
    iced::keyboard::on_key_press(|key, modifiers| match key {
        keyboard::Key::Named(
            keyboard::key::Named::ArrowUp
            | keyboard::key::Named::ArrowDown
            | keyboard::key::Named::ArrowLeft
            | keyboard::key::Named::ArrowRight
            | keyboard::key::Named::Space,
        ) => Some(Message::KeyPressed(key, modifiers)),
        keyboard::Key::Character(ref char_str) => {
            if modifiers.control() {
                match char_str.as_str() {
                    "q" | "Q" => Some(Message::Quit),
                    _ => None,
                }
            } else {
                match char_str.as_str() {
                    "w" | "W" => Some(Message::KeyPressed(
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                        modifiers,
                    )),
                    "a" | "A" => Some(Message::KeyPressed(
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft),
                        modifiers,
                    )),
                    "s" | "S" => Some(Message::KeyPressed(
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown),
                        modifiers,
                    )),
                    "d" | "D" => Some(Message::KeyPressed(
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight),
                        modifiers,
                    )),
                    "p" | "P" => Some(Message::ToggleDarkMode),
                    "z" | "Z" => Some(Message::Undo),
                    _ => None,
                }
            }
        }
        _ => None,
    })
}

fn theme(game: &Game) -> Theme {
    if game.dark_mode {
        Theme::Dark
    } else {
        Theme::Light
    }
}

fn main() -> iced::Result {
    iced::application("2048", update, view)
        .subscription(subscription)
        .theme(theme)
        .run_with(|| {
            let game = Game::load().unwrap_or_else(Game::new);
            (game, Task::none())
        })
}
