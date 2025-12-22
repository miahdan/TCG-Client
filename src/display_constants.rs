use sdl2::pixels::Color;

pub const WINDOW_WIDTH: u32 = 1450;
pub const WINDOW_HEIGHT: u32 = 850;
pub const CARD_LARGE_DISPLAY_WIDTH: u32 = 600;
pub const CARD_LARGE_DISPLAY_HEIGHT: u32 = 850;
pub const CARD_SMALL_DISPLAY_WIDTH: u32 = 60;
pub const CARD_SMALL_DISPLAY_HEIGHT: u32 = 85;

pub const BOARD_SIDE_MARGIN: i32 = 25;

pub const PLAYER1_SLEEVE_COLOR: Color = Color::RGB(255, 0, 0);
pub const PLAYER2_SLEEVE_COLOR: Color = Color::RGB(0, 255, 0);

pub const HIGHLIGHT_COLOR: Color = Color::RGB(0, 200, 200);
pub const SELECTED_COLOR: Color = Color::RGB(0, 0, 255);
pub const HIGHLIGHT_THICKNESS: u32 = 5;

pub const SPACE_BETWEEN_ADJACENT_PRIZES: i32 = 5;

pub const ATTACH_OFFSET_X: i32 = 7;
pub const ATTACH_OFFSET_Y: i32 = 7;

pub const ATTACH_OFFSET_X_ACTIVE: i32 = 15;
pub const ATTACH_OFFSET_Y_ACTIVE: i32 = 0;

pub const PRIZES_BENCH_DISTANCE: i32 = 50;
pub const BENCH_DECK_DISTANCE: i32 = 50;

pub const DECK_DISCARD_DISTANCE: i32 = 60;

pub const HAND_MAT_DISTANCE: i32 = 70;

pub const BENCH_ACTIVE_DISTANCE: i32 = 75;

pub const PRIZES_LOST_ZONE_DISTANCE: i32 = 50;
pub const LOST_ZONE_STADIUM_DISTANCE: i32 = 80;

pub const BENCH_WIDTH: u32 = 485;

pub const HAND_X: i32 = 100;

pub const SEARCH_HIGHLIGHT_HEIGHT: u32 = 145;
pub const SEARCH_UNHIGHLIGHTED_HIGHLIGHTED_DISTANCE: i32 = 50;
pub const SEARCH_UNHIGHLIGHTED_UNHIGHLIGHTED_DISTANCE: i32 = 10;
pub const SEARCH_HIGHLIGHT_COLOR: Color = Color::RGB(200, 200, 0);

pub const DICE_SCALE: u32 = 3;
pub const SPACE_BETWEEN_DICE: u32 = 5;

pub const DICE_ROLL_MAT_COLOR: Color = Color::RGB(100, 100, 100);
pub const DICE_ROLL_MAT_SIDELEN: u32 = 40;

