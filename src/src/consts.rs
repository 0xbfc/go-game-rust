pub const VALID_BOARD_SIZES: &[usize] = &[9, 13, 19];
pub const STAR_POINTS_9X9: &[(usize, usize)] = &[(2, 2), (2, 6), (4, 4), (6, 2), (6, 6)];
pub const STAR_POINTS_13X13: &[(usize, usize)] = &[(3, 3), (3, 9), (6, 6), (9, 3), (9, 9)];
pub const STAR_POINTS_19X19: &[(usize, usize)] = &[
    (3, 3),
    (3, 9),
    (3, 15),
    (9, 3),
    (9, 9),
    (9, 15),
    (15, 3),
    (15, 9),
    (15, 15),
];
pub const DEFAULT_BOARD_SIZE: usize = 19;
pub const CELL_SIZE: f32 = 30.0;
pub const STONE_RADIUS: f32 = 12.0;
pub const TITLE: &str = "Go Game";
pub const WINDOW_SIZE: [f32; 2] = [800.0, 850.0];
