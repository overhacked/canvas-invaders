pub const SHIP_WIDTH: u32 = 48;
pub const SHIP_HEIGHT: u32 = 48;
pub const SHIP: &[u8; 9216] = include_bytes!("windows_pointer.rgba");

pub const BULLET_WIDTH: u32 = 16;
pub const BULLET_HEIGHT: u32 = 16;
pub const BULLET: &[u8; 1024] = include_bytes!("top_side.rgba");

pub const ENEMIES: [&[u8; 1024]; 4] = [ENEMY_LASSO, ENEMY_HOURGLASS, ENEMY_VERTIBEAM, ENEMY_NODROP];

pub const ENEMY_WIDTH: u32 = 16;
pub const ENEMY_HEIGHT: u32 = 16;
pub const ENEMY_LASSO: &[u8; 1024] = include_bytes!("pirate.rgba");
pub const ENEMY_HOURGLASS: &[u8; 1024] = include_bytes!("wait-01.rgba");
pub const ENEMY_VERTIBEAM: &[u8; 1024] = include_bytes!("vertical-text.rgba");
pub const ENEMY_NODROP: &[u8; 1024] = include_bytes!("dnd-no-drop.rgba");
