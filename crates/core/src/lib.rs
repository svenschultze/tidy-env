pub mod object;
pub mod gen;
pub mod agent;
pub mod sim;

use gen::Cell;
pub const OUTSIDE: Cell = -1;
pub const WALL: Cell = -2;
pub const CLOSED_DOOR: Cell = -3;
pub const OPEN_DOOR: Cell = -4;

pub const OBSTACLES: [Cell; 3] = [OUTSIDE, WALL, CLOSED_DOOR];

pub use object::{Object, ObjectType, ObjectSchema, ObjectId};
pub use gen::{GenOpts, Layout, World, generate};
pub use agent::Agent;
pub use sim::{Simulator, MoveError};