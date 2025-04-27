// wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use core as apartment_core;    // assumes your core crate’s Cargo.toml name is “core”

#[wasm_bindgen]
pub struct ApartmentLayout {
    width: usize,
    height: usize,
    /// Linear row-major i8 grid: -3=outside, -1=wall, -2=door, 0..=room IDs
    cells: Vec<i8>,
}

#[wasm_bindgen]
impl ApartmentLayout {
    /// Number of columns
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.width
    }
    /// Number of rows
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.height
    }
    /// Flat array of length width*height
    #[wasm_bindgen(getter)]
    pub fn cells(&self) -> Vec<i8> {
        self.cells.clone()
    }
}

#[wasm_bindgen]
pub fn generate(seed: u64, max_rooms: usize) -> ApartmentLayout {
    // Build the core::GenOpts
    let opts = apartment_core::GenOpts { seed, max_rooms };

    // Call into your core crate
    let core::Layout { width, height, cells } = apartment_core::generate(&opts);

    // Wrap it up for JS
    ApartmentLayout { width, height, cells }
}

#[wasm_bindgen]
/// Simulator wrapper exposing agent movement API
pub struct ApartmentSimulator {
    sim: apartment_core::Simulator,
}

#[wasm_bindgen]
impl ApartmentSimulator {
    /// Create a new simulator with given seed, max_rooms, and start coordinates
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64, max_rooms: usize, start_x: usize, start_y: usize) -> Result<ApartmentSimulator, JsValue> {
        let opts = apartment_core::GenOpts { seed, max_rooms };
        // generate layout
        let layout = apartment_core::generate(&opts);
        match apartment_core::Simulator::new(layout, start_x, start_y) {
            Ok(sim) => Ok(ApartmentSimulator { sim }),
            Err(e)  => Err(JsValue::from_str(e)),
        }
    }

    /// Current agent X coordinate
    #[wasm_bindgen(getter)]
    pub fn agent_x(&self) -> usize {
        self.sim.agent.x
    }
    /// Current agent Y coordinate
    #[wasm_bindgen(getter)]
    pub fn agent_y(&self) -> usize {
        self.sim.agent.y
    }

    /// Move agent up (y-1)
    #[wasm_bindgen]
    pub fn up(&mut self) -> Result<(), JsValue> {
        self.sim.up().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }
    /// Move agent down (y+1)
    #[wasm_bindgen]
    pub fn down(&mut self) -> Result<(), JsValue> {
        self.sim.down().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }
    /// Move agent left (x-1)
    #[wasm_bindgen]
    pub fn left(&mut self) -> Result<(), JsValue> {
        self.sim.left().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }
    /// Move agent right (increasing x)
    #[wasm_bindgen]
    pub fn right(&mut self) -> Result<(), JsValue> {
        self.sim.right().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Open a closed door in the given direction relative to the agent
    #[wasm_bindgen]
    pub fn interact(&mut self, dx: i32, dy: i32) -> Result<(), JsValue> {
        self.sim
            .interact(dx as isize, dy as isize)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }
    /// Convenience: open door above
    #[wasm_bindgen]
    pub fn open_up(&mut self) -> Result<(), JsValue> { self.interact(0, -1) }
    /// Convenience: open door below
    #[wasm_bindgen]
    pub fn open_down(&mut self) -> Result<(), JsValue> { self.interact(0, 1) }
    /// Convenience: open door left
    #[wasm_bindgen]
    pub fn open_left(&mut self) -> Result<(), JsValue> { self.interact(-1, 0) }
    /// Convenience: open door right
    #[wasm_bindgen]
    pub fn open_right(&mut self) -> Result<(), JsValue> { self.interact(1, 0) }

    /// Layout width
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.sim.layout.width
    }
    /// Layout height
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.sim.layout.height
    }
    /// Flat cells array
    #[wasm_bindgen(getter)]
    pub fn cells(&self) -> Vec<i8> {
        self.sim.layout.cells.clone()
    }
}
