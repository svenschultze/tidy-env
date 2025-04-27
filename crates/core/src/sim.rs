use crate::agent::Agent;
use crate::{World, Object, ObjectId};

/// Constants for door types
use crate::{
    CLOSED_DOOR, OPEN_DOOR, OBSTACLES,
};

/// Errors when attempting agent movements or object actions
#[derive(Debug, PartialEq)]
pub enum MoveError {
    /// Move would go outside layout bounds
    OutOfBounds,
    /// Move would hit a non-navigable cell (wall or outside)
    HitObstacle,
    /// Agent already holding an object
    AlreadyHolding,
    /// No pickable object at agent location
    NothingToPickUp,
    /// Agent not holding any object
    NotHolding,
    /// Container is full
    ContainerFull,
    /// Target invalid or not a container
    InvalidTarget,
}

/// Simulator state pairing a world with an agent and optional held object
#[derive(Debug)]
pub struct Simulator {
    pub world: World,
    pub agent: Agent,
    pub holding: Option<Object>,
}

impl Simulator {
    /// Initialize simulator with world and agent start
    pub fn new(world: World, start_x: usize, start_y: usize) -> Result<Self, &'static str> {
        // verify layout dimensions
        let layout = &world.layout;
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
        Ok(Simulator { world, agent, holding: None })
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
            || new_x >= self.world.layout.width as isize
            || new_y < 0
            || new_y >= self.world.layout.height as isize
        {
            return Err(MoveError::OutOfBounds);
        }
        let new_x = new_x as usize;
        let new_y = new_y as usize;
        let idx = new_y * self.world.layout.width + new_x;
        // obstacle or closed door
        if OBSTACLES.contains(&self.world.layout.cells[idx]) {
            return Err(MoveError::HitObstacle);
        }
        self.agent.x = new_x;
        self.agent.y = new_y;
        Ok(())
    }

    /// Unified interact: doors and objects both handled at target cell
    pub fn interact(&mut self, dx: isize, dy: isize) -> Result<(), String> {
        // target coordinates
        let tx_i = self.agent.x as isize + dx;
        let ty_i = self.agent.y as isize + dy;
        // bounds check
        if tx_i < 0 || tx_i >= self.world.layout.width as isize || ty_i < 0 || ty_i >= self.world.layout.height as isize {
            return Err(format!("Out of bounds: ({}, {})", tx_i, ty_i));
        }
        let tx = tx_i as usize;
        let ty = ty_i as usize;
        let idx = ty * self.world.layout.width + tx;
        let cell_value = self.world.layout.cells[idx];
        // door handling
        if cell_value == CLOSED_DOOR {
            self.use_door(tx_i, ty_i, true)?;
            return Ok(());
        }
        if cell_value == OPEN_DOOR {
            self.use_door(tx_i, ty_i, false)?;
            return Ok(());
        }
        // only allow object interactions on room cells (>=0)
        if cell_value < 0 {
            return Err(format!("Cannot interact with objects on non-room cell: {}", cell_value));
        }
        // object interaction at target
        if self.holding.is_none() {
            // pick up if pickable present
            if let Some(pos) = self.world.objects.iter().position(|o| o.x == tx && o.y == ty && o.pickable) {
                let obj = self.world.objects.remove(pos);
                self.holding = Some(obj);
                return Ok(());
            }
            return Err("Nothing to interact with".into());
        } else {
            // try placing into container at target
            if let Some(container) = self.world.objects.iter().find(|o| o.x == tx && o.y == ty) {
                if self.place_into(container.id).is_ok() {
                    return Ok(());
                }
            }
            // drop at target
            let mut obj = self.holding.take().unwrap();
            obj.x = tx;
            obj.y = ty;
            self.world.objects.push(obj);
            return Ok(());
        }
    }

    /// Use a door (open if `open_flag` is true, close otherwise)
    pub fn use_door(&mut self, x: isize, y: isize, open_flag: bool) -> Result<(), String> {
        // bounds
        if x < 0
            || x >= self.world.layout.width as isize
            || y < 0
            || y >= self.world.layout.height as isize
        {
            return Err(format!("Out of bounds: ({}, {})", x, y));
        }
        let idx = (y as usize) * self.world.layout.width + (x as usize);
        let change_to = if open_flag {
            OPEN_DOOR
        } else {
            CLOSED_DOOR
        };
        let cell_value = self.world.layout.cells[idx];

        // for every connected cell to x, y with the same value as cell_value, change it to change_to with flood fill
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let idx = (cy as usize) * self.world.layout.width + (cx as usize);
            if self.world.layout.cells[idx] == cell_value {
                self.world.layout.cells[idx] = change_to;
                // push all 4 neighbors
                stack.push((cx - 1, cy));
                stack.push((cx + 1, cy));
                stack.push((cx, cy - 1));
                stack.push((cx, cy + 1));
            }
        }
        
        Ok(())
    }

    /// Pick up a pickable object at the agent's current location
    pub fn pick_up(&mut self) -> Result<(), MoveError> {
        if self.holding.is_some() {
            return Err(MoveError::AlreadyHolding);
        }
        let (ax, ay) = (self.agent.x, self.agent.y);
        if let Some(pos) = self.world.objects.iter().position(|o| o.x == ax && o.y == ay && o.pickable) {
            let obj = self.world.objects.remove(pos);
            self.holding = Some(obj);
            Ok(())
        } else {
            Err(MoveError::NothingToPickUp)
        }
    }

    /// Drop held object at the agent's current location or into the world
    pub fn drop(&mut self) -> Result<(), MoveError> {
        if let Some(mut obj) = self.holding.take() {
            obj.x = self.agent.x;
            obj.y = self.agent.y;
            self.world.objects.push(obj);
            Ok(())
        } else {
            Err(MoveError::NotHolding)
        }
    }

    /// Place held object into a container object
    pub fn place_into(&mut self, target_id: ObjectId) -> Result<(), MoveError> {
        if self.holding.is_none() {
            return Err(MoveError::NotHolding);
        }
        // find container
        if let Some(container) = self.world.objects.iter_mut().find(|o| o.id == target_id) {
            // determine capacity
            let capacity = match container.typ {
                crate::object::ObjectType::Wardrobe { capacity } => capacity,
                crate::object::ObjectType::Cupboard { capacity } => capacity,
                _ => return Err(MoveError::InvalidTarget),
            };
            if container.contents.len() >= capacity {
                return Err(MoveError::ContainerFull);
            }
            // place object
            let mut obj = self.holding.take().unwrap();
            obj.x = container.x;
            obj.y = container.y;
            container.contents.push(obj.id);
            self.world.objects.push(obj);
            Ok(())
        } else {
            Err(MoveError::InvalidTarget)
        }
    }
}
