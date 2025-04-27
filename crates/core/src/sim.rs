use crate::gen::Layout;
use crate::agent::Agent;

/// Errors when attempting agent movements
#[derive(Debug, PartialEq)]
pub enum MoveError {
    /// Move would go outside layout bounds
    OutOfBounds,
    /// Move would hit a non-navigable cell (wall or outside)
    HitObstacle,
}

/// Simulator state pairing a layout with an agent
#[derive(Debug)]
pub struct Simulator {
    pub layout: Layout,
    pub agent: Agent,
}

/// Constants for door types
use crate::{
    CLOSED_DOOR, OPEN_DOOR, OBSTACLES,
};

impl Simulator {
    /// Initialize simulator with layout and agent start
    pub fn new(layout: Layout, start_x: usize, start_y: usize) -> Result<Self, &'static str> {
        // verify layout dimensions
        if layout.cells.len() != layout.width * layout.height {
            return Err("Layout cells length does not match dimensions");
        }
        // bounds check
        if start_x >= layout.width || start_y >= layout.height {
            return Err("Start position out of bounds");
        }
        let idx = start_y * layout.width + start_x;
        if layout.cells[idx] < 0 {
            return Err("Start position is not navigable");
        }
        let agent = Agent::new(start_x, start_y);
        Ok(Simulator { layout, agent })
    }

    /// Move agent up
    pub fn up(&mut self) -> Result<(), MoveError> {
        self.try_move(0, -1)
    }

    /// Move agent down
    pub fn down(&mut self) -> Result<(), MoveError> {
        self.try_move(0, 1)
    }

    /// Move agent left
    pub fn left(&mut self) -> Result<(), MoveError> {
        self.try_move(-1, 0)
    }

    /// Move agent right
    pub fn right(&mut self) -> Result<(), MoveError> {
        self.try_move(1, 0)
    }

    fn try_move(&mut self, dx: isize, dy: isize) -> Result<(), MoveError> {
        let new_x = self.agent.x as isize + dx;
        let new_y = self.agent.y as isize + dy;
        // bounds
        if new_x < 0
            || new_x >= self.layout.width as isize
            || new_y < 0
            || new_y >= self.layout.height as isize
        {
            return Err(MoveError::OutOfBounds);
        }
        let new_x = new_x as usize;
        let new_y = new_y as usize;
        let idx = new_y * self.layout.width + new_x;
        // obstacle or closed door
        if OBSTACLES.contains(&self.layout.cells[idx]) {
            return Err(MoveError::HitObstacle);
        }
        self.agent.x = new_x;
        self.agent.y = new_y;
        Ok(())
    }

    /// Interact with a door adjacent to the agent at (dx, dy)
    pub fn interact(&mut self, dx: isize, dy: isize) -> Result<(), String> {
        let new_x = self.agent.x as isize + dx;
        let new_y = self.agent.y as isize + dy;
        // bounds
        if new_x < 0
            || new_x >= self.layout.width as isize
            || new_y < 0
            || new_y >= self.layout.height as isize
        {
            return Err(format!("Out of bounds: ({}, {})", new_x, new_y));
        }
        let idx = (new_y as usize) * self.layout.width + (new_x as usize);
        let cell_value = self.layout.cells[idx];
        match cell_value {
            CLOSED_DOOR => {
                // open the door
                self.use_door(new_x, new_y, true)?;
                Ok(())
            }
            OPEN_DOOR => {
                // close the door
                self.use_door(new_x, new_y, false)?;
                Ok(())
            }
            _ => Err(format!("Not a door: {}", cell_value)),
        }
    }

    /// Use a door (open if `open_flag` is true, close otherwise)
    pub fn use_door(&mut self, x: isize, y: isize, open_flag: bool) -> Result<(), String> {
        // bounds
        if x < 0
            || x >= self.layout.width as isize
            || y < 0
            || y >= self.layout.height as isize
        {
            return Err(format!("Out of bounds: ({}, {})", x, y));
        }
        let idx = (y as usize) * self.layout.width + (x as usize);
        let change_to = if open_flag {
            OPEN_DOOR
        } else {
            CLOSED_DOOR
        };
        let cell_value = self.layout.cells[idx];

        // for every connected cell to x, y with the same value as cell_value, change it to change_to with flood fill
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let idx = (cy as usize) * self.layout.width + (cx as usize);
            if self.layout.cells[idx] == cell_value {
                self.layout.cells[idx] = change_to;
                // push all 4 neighbors
                stack.push((cx - 1, cy));
                stack.push((cx + 1, cy));
                stack.push((cx, cy - 1));
                stack.push((cx, cy + 1));
            }
        }
        
        Ok(())
    }
}
