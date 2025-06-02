pub mod object;
pub mod gen;
pub mod agent;
pub mod sim;
pub mod language;

pub use object::{Object, ObjectSchema, ObjectId};
pub use gen::{GenOpts, Layout, World, generate};
pub use agent::Agent;
pub use sim::{Simulator, MoveError};

// Cell type constants
pub const WALL: i8 = -1;
pub const OUTSIDE: i8 = -2;
pub const CLOSED_DOOR: i8 = -3;
pub const OPEN_DOOR: i8 = -4;
pub const OBSTACLES: [i8; 3] = [WALL, OUTSIDE, CLOSED_DOOR];