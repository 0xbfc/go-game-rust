use eframe::egui;
use std::collections::HashSet;
use std::io;
mod consts;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stone {
    Black,
    White,
    Empty,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Player {
    Black,
    White,
}

impl Player {
    fn other(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
    fn to_stone(&self) -> Stone {
        match self {
            Player::Black => Stone::Black,
            Player::White => Stone::White,
        }
    }
}

struct GoBoard {
    board_size: usize,
    board: Vec<Vec<Stone>>,
    current_player: Player,
    captured_black: u32,
    captured_white: u32,
    game_over: bool,
    last_move: Option<(usize, usize)>,
}

impl Default for GoBoard {
    fn default() -> Self {
        Self {
            board_size: consts::DEFAULT_BOARD_SIZE,
            board: vec![vec![Stone::Empty; consts::DEFAULT_BOARD_SIZE]; consts::DEFAULT_BOARD_SIZE],
            current_player: Player::Black,
            captured_black: 0,
            captured_white: 0,
            game_over: false,
            last_move: None,
        }
    }
}

impl GoBoard {
    fn _new() -> Self {
        Self::default()
    }

    fn with_size(board_size_param: usize) -> Self {
        GoBoard {
            board_size: board_size_param,
            board: vec![vec![Stone::Empty; board_size_param]; board_size_param],
            current_player: Player::Black,
            captured_black: 0,
            captured_white: 0,
            game_over: false,
            last_move: None,
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in directions.iter() {
            let new_row = row as i32 + dr;
            let new_col = col as i32 + dc;
            if new_row >= 0
                && new_row < self.board_size as i32
                && new_col >= 0
                && new_col < self.board_size as i32
            {
                neighbors.push((new_row as usize, new_col as usize));
            }
        }
        neighbors
    }

    fn get_group(&self, row: usize, col: usize, stone: Stone) -> HashSet<(usize, usize)> {
        let mut group = HashSet::new();
        let mut stack = vec![(row, col)];
        while let Some((r, c)) = stack.pop() {
            if group.contains(&(r, c)) || self.board[r][c] != stone {
                continue;
            }
            group.insert((r, c));
            for (nr, nc) in self.get_neighbors(r, c) {
                if !group.contains(&(nr, nc)) && self.board[nr][nc] == stone {
                    stack.push((nr, nc));
                }
            }
        }
        group
    }

    fn has_liberties(&self, row: usize, col: usize) -> bool {
        let stone = self.board[row][col];
        if stone == Stone::Empty {
            return true;
        }
        let group = self.get_group(row, col, stone);
        for &(r, c) in &group {
            for (nr, nc) in self.get_neighbors(r, c) {
                if self.board[nr][nc] == Stone::Empty {
                    return true;
                }
            }
        }
        false
    }

    fn capture_stones(&mut self, opponent: Stone) -> u32 {
        let mut captured = 0;
        let mut to_remove = Vec::new();
        for row in 0..self.board_size {
            for col in 0..self.board_size {
                if self.board[row][col] == opponent && !self.has_liberties(row, col) {
                    let group = self.get_group(row, col, opponent);
                    for &(r, c) in &group {
                        to_remove.push((r, c));
                    }
                    captured += group.len() as u32;
                }
            }
        }
        for (r, c) in to_remove {
            self.board[r][c] = Stone::Empty;
        }
        captured
    }

    fn would_capture_opponent(&self, row: usize, col: usize, player: Player) -> bool {
        let opponent_stone = player.other().to_stone();
        for (nr, nc) in self.get_neighbors(row, col) {
            if self.board[nr][nc] == opponent_stone {
                // Check if this opponent group would have no liberties after our move
                if self.would_group_be_captured(nr, nc, opponent_stone, row, col) {
                    return true;
                }
            }
        }
        false
    }
    fn would_group_be_captured(
        &self,
        group_row: usize,
        group_col: usize,
        group_stone: Stone,
        new_stone_row: usize,
        new_stone_col: usize,
    ) -> bool {
        let group = self.get_group(group_row, group_col, group_stone);
        for &(r, c) in &group {
            for (nr, nc) in self.get_neighbors(r, c) {
                // If there's an empty liberty that's not where we're placing our stone
                if self.board[nr][nc] == Stone::Empty
                    && !(nr == new_stone_row && nc == new_stone_col)
                {
                    return false;
                }
            }
        }
        true // No liberties found
    }
    fn would_be_suicide(&self, row: usize, col: usize, player: Player) -> bool {
        let player_stone = player.to_stone();
        // Check if placing the stone would create a group with no liberties
        // First, check direct liberties (empty adjacent spots)
        for (nr, nc) in self.get_neighbors(row, col) {
            if self.board[nr][nc] == Stone::Empty {
                return false; // Has at least one liberty
            }
        }
        // Check if we can connect to a friendly group that has liberties
        for (nr, nc) in self.get_neighbors(row, col) {
            if self.board[nr][nc] == player_stone {
                // Check if this friendly group would still have liberties after our move
                if self.would_friendly_group_have_liberties(nr, nc, player_stone, row, col) {
                    return false;
                }
            }
        }
        true // Would be suicide
    }
    fn would_friendly_group_have_liberties(
        &self,
        group_row: usize,
        group_col: usize,
        group_stone: Stone,
        new_row: usize,
        new_col: usize,
    ) -> bool {
        let group = self.get_group(group_row, group_col, group_stone);
        for &(r, c) in &group {
            for (nr, nc) in self.get_neighbors(r, c) {
                // Check for empty spots (but not where we're placing the new stone)
                if self.board[nr][nc] == Stone::Empty && !(nr == new_row && nc == new_col) {
                    return true;
                }
            }
        }
        // Also check the new stone's position for additional liberties
        for (nr, nc) in self.get_neighbors(new_row, new_col) {
            if self.board[nr][nc] == Stone::Empty {
                return true;
            }
        }
        false
    }

    fn is_valid_move(&self, row: usize, col: usize) -> bool {
        if self.game_over || self.board[row][col] != Stone::Empty {
            return false;
        }
        // Check if the move would capture opponent stones
        let would_capture = self.would_capture_opponent(row, col, self.current_player);
        // If we wouldn't capture anything, check if it would be suicide
        if !would_capture && self.would_be_suicide(row, col, self.current_player) {
            return false;
        }
        true
    }

    fn make_move(&mut self, row: usize, col: usize) -> bool {
        if !self.is_valid_move(row, col) {
            return false;
        }
        self.board[row][col] = self.current_player.to_stone();
        self.last_move = Some((row, col));
        // Capture opponent stones
        let opponent_stone = self.current_player.other().to_stone();
        let captured = self.capture_stones(opponent_stone);
        match self.current_player {
            Player::Black => self.captured_white += captured,
            Player::White => self.captured_black += captured,
        }
        self.current_player = self.current_player.other();
        true
    }

    fn pass_turn(&mut self) {
        self.current_player = self.current_player.other();
    }
}

impl eframe::App for GoBoard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Go Game");
            ui.horizontal(|ui| {
                ui.label(format!("Current Player: {:?}", self.current_player));
                ui.separator();
                ui.label(format!(
                    "Captured - Black: {}, White: {}",
                    self.captured_black, self.captured_white
                ));
                if ui.button("Pass").clicked() {
                    self.pass_turn();
                }
                if ui.button("Reset Game").clicked() {
                    self.reset();
                }
            });
            ui.separator();
            // Calculate board dimensions
            let board_size = consts::CELL_SIZE * (self.board_size as f32 + 1.0);
            let (response, painter) =
                ui.allocate_painter(egui::Vec2::splat(board_size), egui::Sense::click());
            let board_rect = response.rect;
            let top_left = board_rect.min + egui::Vec2::splat(consts::CELL_SIZE * 0.5);
            // Draw grid lines
            let line_color = egui::Color32::from_rgb(101, 67, 33);
            for i in 0..self.board_size {
                let offset = i as f32 * consts::CELL_SIZE;
                // Horizontal lines
                painter.line_segment(
                    [
                        top_left + egui::Vec2::new(0.0, offset),
                        top_left
                            + egui::Vec2::new((self.board_size - 1) as f32 * consts::CELL_SIZE, offset),
                    ],
                    egui::Stroke::new(1.0, line_color),
                );
                // Vertical lines
                painter.line_segment(
                    [
                        top_left + egui::Vec2::new(offset, 0.0),
                        top_left
                            + egui::Vec2::new(offset, (self.board_size - 1) as f32 * consts::CELL_SIZE),
                    ],
                    egui::Stroke::new(1.0, line_color),
                );
            }
            // Draw star points (handicap points)
            let star_points: &[(usize, usize)];
            if self.board_size == consts::VALID_BOARD_SIZES[0] {
                star_points = consts::STAR_POINTS_9X9;
            } else if self.board_size == consts::VALID_BOARD_SIZES[1] {
                star_points = consts::STAR_POINTS_13X13;
            } else {
                star_points = consts::STAR_POINTS_19X19;
            }

            for &(row, col) in star_points {
                let pos =
                    top_left + egui::Vec2::new(col as f32 * consts::CELL_SIZE, row as f32 * consts::CELL_SIZE);
                painter.circle_filled(pos, 3.0, line_color);
            }
            // Draw stones
            for row in 0..self.board_size {
                for col in 0..self.board_size {
                    let stone = self.board[row][col];
                    if stone != Stone::Empty {
                        let pos = top_left
                            + egui::Vec2::new(col as f32 * consts::CELL_SIZE, row as f32 * consts::CELL_SIZE);
                        let stone_color = match stone {
                            Stone::Black => egui::Color32::BLACK,
                            Stone::White => egui::Color32::WHITE,
                            Stone::Empty => continue,
                        };
                        // Draw stone shadow
                        painter.circle_filled(
                            pos + egui::Vec2::new(1.0, 1.0),
                            consts::STONE_RADIUS,
                            egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
                        );
                        // Draw stone
                        painter.circle_filled(pos, consts::STONE_RADIUS, stone_color);
                        // Draw stone border
                        painter.circle_stroke(
                            pos,
                            consts::STONE_RADIUS,
                            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                        );
                        // Highlight last move
                        if let Some((last_row, last_col)) = self.last_move {
                            if row == last_row && col == last_col {
                                painter.circle_stroke(
                                    pos,
                                    consts::STONE_RADIUS + 3.0,
                                    egui::Stroke::new(2.0, egui::Color32::RED),
                                );
                            }
                        }
                    }
                }
            }
            // Handle clicks
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let rel_pos = pos - top_left;
                    let col = ((rel_pos.x + consts::CELL_SIZE * 0.5) / consts::CELL_SIZE) as usize;
                    let row = ((rel_pos.y + consts::CELL_SIZE * 0.5) / consts::CELL_SIZE) as usize;
                    if row < self.board_size && col < self.board_size {
                        self.make_move(row, col);
                    }
                }
            }
            // Show move validity hint
            if let Some(hover_pos) = response.hover_pos() {
                let rel_pos = hover_pos - top_left;
                let col = ((rel_pos.x + consts::CELL_SIZE * 0.5) / consts::CELL_SIZE) as usize;
                let row = ((rel_pos.y + consts::CELL_SIZE * 0.5) / consts::CELL_SIZE) as usize;
                if row < self.board_size
                    && col < self.board_size
                    && self.board[row][col] == Stone::Empty
                {
                    let pos =
                        top_left + egui::Vec2::new(col as f32 * consts::CELL_SIZE, row as f32 * consts::CELL_SIZE);
                    let is_valid = self.is_valid_move(row, col);
                    let preview_color = match self.current_player {
                        Player::Black => egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
                        Player::White => egui::Color32::from_rgba_premultiplied(255, 255, 255, 150),
                    };
                    if is_valid {
                        painter.circle_filled(pos, consts::STONE_RADIUS * 0.7, preview_color);
                    }
                }
            }
        });
    }
}

fn get_board_size(prompt: &str) -> usize {
    loop {
        println!("{}", prompt);

        let mut input = String::new();

        // Read input from stdin
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();

                // Check if input is empty (user pressed Enter)
                if trimmed.is_empty() {
                    println!("Proceeding with default board size.");
                    return consts::DEFAULT_BOARD_SIZE;
                }

                // Try to parse the input as an integer
                match trimmed.parse::<usize>() {
                    Ok(number) => {
                        if consts::VALID_BOARD_SIZES.contains(&number) {
                            return number;
                        } else {
                            println!("Invalid input. Please enter a valid integer.")
                        }
                    }
                    Err(_) => println!("Invalid input. Please enter a valid integer."),
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                continue;
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let inputted_board_size =
        get_board_size("Please enter a board size of 9, 13, or 19 (press Enter for 19): ");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(consts::WINDOW_SIZE)
            .with_title(consts::TITLE),
        ..Default::default()
    };
    eframe::run_native(
        consts::TITLE,
        options,
        Box::new(|_cc| Ok(Box::new(GoBoard::with_size(inputted_board_size)))),
    )
}
