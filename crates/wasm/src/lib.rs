// wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use core as apartment_core;    // assumes your core crate’s Cargo.toml name is “core”
use js_sys::{Array, Object as JsObject, Reflect};
use wasm_bindgen::JsValue;

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
    let world = apartment_core::generate(&opts);
    let core::Layout { width, height, cells } = world.layout;

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
    
    /// Pick up a pickable object at the agent's location
    #[wasm_bindgen]
    pub fn pick_up(&mut self) -> Result<(), JsValue> {
        self.sim.pick_up().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }
    /// Drop held object at the agent's location
    #[wasm_bindgen]
    pub fn drop(&mut self) -> Result<(), JsValue> {
        self.sim.drop().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Retrieve all objects in the world
    #[wasm_bindgen]
    pub fn get_objects(&self) -> Array {
        let arr = Array::new();
        // collect IDs of objects contained in containers
        use std::collections::HashSet;
        let mut contained: HashSet<u32> = HashSet::new();
        for c in self.sim.world.objects.iter() {
            for &cid in c.contents.iter() {
                contained.insert(cid as u32);
            }
        }
        for o in self.sim.world.objects.iter() {
            // skip objects inside containers
            if contained.contains(&(o.id as u32)) {
                continue;
            }
            let obj = JsObject::new();
            // set type string
            let type_str = match &o.typ {
                apartment_core::ObjectType::Wardrobe { .. } => "Wardrobe",
                apartment_core::ObjectType::Cupboard { .. } => "Cupboard",
                apartment_core::ObjectType::Banana => "Banana",
                apartment_core::ObjectType::Couch => "Couch",
                _ => "Unknown",
            };
            Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(o.id as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("x"), &JsValue::from_f64(o.x as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("y"), &JsValue::from_f64(o.y as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("pickable"), &JsValue::from_bool(o.pickable)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str(type_str)).unwrap();
            arr.push(&obj);
        }
        arr
    }

    /// Layout width
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.sim.world.layout.width
    }
    /// Layout height
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> usize {
        self.sim.world.layout.height
    }
    /// Flat cells array
    #[wasm_bindgen(getter)]
    pub fn cells(&self) -> Vec<i8> {
        self.sim.world.layout.cells.clone()
    }
}
