use crate::gen::Layout;

/// Possible errors when moving the agent
#[derive(Debug, PartialEq)]
pub enum MoveError {
    /// Attempted to move outside the layout bounds
    OutOfBounds,
    /// Attempted to move into a wall or outside cell
    HitObstacle,
}

/// Represents an agent within the layout
#[derive(Debug)]
pub struct Agent {
    pub x: usize,
    pub y: usize,
}

impl Agent {
    /// Create a new agent at the given coordinates
    pub fn new(x: usize, y: usize) -> Self {
        Agent { x, y }
    }
}

/// Simulator state pairing a layout with an agent
#[derive(Debug)]
pub struct Simulator {
    pub layout: Layout,
    pub agent: Agent,
}

impl Simulator {
    /// Initialize the simulator with a layout and agent start position
    pub fn new(layout: Layout, start_x: usize, start_y: usize) -> Result<Self, &'static str> {
        // Check bounds
        if start_x >= layout.width || start_y >= layout.height {
            return Err("Start position out of bounds");
        }
        // Check starting cell is free (non-negative)
        let idx = start_y * layout.width + start_x;
        if layout.cells[idx] < 0 {
            return Err("Start position is not navigable");
        }
        Ok(Simulator {
            layout,
            agent: Agent::new(start_x, start_y),
        })
    }

    /// Attempt to move the agent up (decreasing y)
    pub fn up(&mut self) -> Result<(), MoveError> {
        self.try_move(0, -1)
    }

    /// Move the agent down (increasing y)
    pub fn down(&mut self) -> Result<(), MoveError> {
        self.try_move(0, 1)
    }

    /// Move the agent left (decreasing x)
    pub fn left(&mut self) -> Result<(), MoveError> {
        self.try_move(-1, 0)
    }

    /// Move the agent right (increasing x)
    pub fn right(&mut self) -> Result<(), MoveError> {
        self.try_move(1, 0)
    }

    /// Internal helper: attempt to move by (dx, dy)
    fn try_move(&mut self, dx: isize, dy: isize) -> Result<(), MoveError> {
        let new_x = self.agent.x as isize + dx;
        let new_y = self.agent.y as isize + dy;
        // Bounds check
        if new_x < 0 || new_x >= self.layout.width as isize || new_y < 0 || new_y >= self.layout.height as isize {
            return Err(MoveError::OutOfBounds);
        }
        let new_x = new_x as usize;
        let new_y = new_y as usize;
        let idx = new_y * self.layout.width + new_x;
        // Obstacle check (walls and outside are negative)
        if self.layout.cells[idx] < 0 {
            return Err(MoveError::HitObstacle);
        }
        // Apply move
        self.agent.x = new_x;
        self.agent.y = new_y;
        Ok(())
    }
}
