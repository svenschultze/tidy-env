pub mod gen;
pub mod agent;
pub mod sim;

use gen::Cell;
pub const OUTSIDE: Cell = -1;
pub const WALL: Cell = -2;
pub const CLOSED_DOOR: Cell = -3;
pub const OPEN_DOOR: Cell = -4;
pub const OBJECT: Cell = -5;

pub const OBSTACLES: [Cell; 4] = [OUTSIDE, WALL, CLOSED_DOOR, OBJECT];

pub use gen::{GenOpts, Layout, generate};
pub use agent::Agent;
pub use sim::{Simulator, MoveError};