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
/// Generate a new layout with custom width/height
pub fn generate(seed: u64, max_rooms: usize, width: usize, height: usize, max_objects: usize) -> ApartmentLayout {
    let opts = apartment_core::GenOpts { seed, max_rooms, width, height, max_objects };

    // Call into your core crate
    let world = apartment_core::generate(&opts);
    let layout = world.layout;

    // Wrap it up for JS (ignore room_names here; use get_room_names method)
    ApartmentLayout { width: layout.width, height: layout.height, cells: layout.cells }
}

// Expose room names on the layout
#[wasm_bindgen]
impl ApartmentLayout {
    /// Names of each room ID (0..rooms.len())
    #[wasm_bindgen]
    pub fn get_room_names(&self) -> Array {
        // need to call into core: regenerate? Instead, use simulator get_room_names after instantiation
        Array::new()
    }
}

#[wasm_bindgen]
/// Simulator wrapper exposing agent movement API
pub struct ApartmentSimulator {
    sim: apartment_core::Simulator,
}

#[wasm_bindgen]
impl ApartmentSimulator {
    #[wasm_bindgen(constructor)]
    /// Create a new simulator with custom width, height (agent start auto-selected)
    pub fn new(seed: u64, max_rooms: usize, width: usize, height: usize, max_objects: usize) -> Result<ApartmentSimulator, JsValue> {
        let opts = apartment_core::GenOpts { seed, max_rooms, width, height, max_objects };
        let world = apartment_core::generate(&opts);
        // auto-select first room cell
        let mut start_x = 0;
        let mut start_y = 0;
        for (i, &cell) in world.layout.cells.iter().enumerate() {
            if cell >= 0 {
                start_x = i % world.layout.width;
                start_y = i / world.layout.width;
                break;
            }
        }
        apartment_core::Simulator::new(world, start_x, start_y)
            .map(|sim| ApartmentSimulator { sim })
            .map_err(|e| JsValue::from_str(e))
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
        // use std::collections::HashSet;
        // let mut contained: HashSet<u32> = HashSet::new();
        // for c in self.sim.world.objects.iter() {
        //     for &cid in c.contents.iter() {
        //         contained.insert(cid as u32);
        //     }
        // }
        for o in self.sim.world.objects.iter() {
            // skip objects inside containers
            // if contained.contains(&(o.id as u32)) {
            //     continue;
            // }
            let obj = JsObject::new();
            Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(o.id as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("x"), &JsValue::from_f64(o.x as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("y"), &JsValue::from_f64(o.y as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("pickable"), &JsValue::from_bool(o.pickable)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(o.name)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("capacity"), &JsValue::from_f64(o.capacity as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("description"), &JsValue::from_str(o.description)).unwrap();
            let contents_arr = Array::new();
            for &c in o.contents.iter() {
                contents_arr.push(&JsValue::from_f64(c as f64));
            }
            Reflect::set(&obj, &JsValue::from_str("contents"), &contents_arr).unwrap();
            arr.push(&obj);
        }
        arr
    }
    /// Get the object currently held by the agent (or null)
    #[wasm_bindgen]
    pub fn get_holding(&self) -> JsValue {
        if let Some(o) = &self.sim.holding {
            let obj = JsObject::new();
            Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(o.id as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("pickable"), &JsValue::from_bool(o.pickable)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(o.name)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("capacity"), &JsValue::from_f64(o.capacity as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("description"), &JsValue::from_str(o.description)).unwrap();
            let contents_arr = Array::new();
            for &c in o.contents.iter() {
                contents_arr.push(&JsValue::from_f64(c as f64));
            }
            Reflect::set(&obj, &JsValue::from_str("contents"), &contents_arr).unwrap();
            JsValue::from(obj)
        } else {
            JsValue::NULL
        }
    }
    /// Get contents of a container object by ID
    #[wasm_bindgen]
    pub fn get_contents(&self, container_id: u32) -> Array {
        let arr = Array::new();
        // find container
        if let Some(container) = self.sim.world.objects.iter().find(|o| o.id as u32 == container_id) {
            for &cid in container.contents.iter() {
                if let Some(inner) = self.sim.world.objects.iter().find(|o| o.id == cid) {
                    let obj = JsObject::new();
                    Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(inner.id as f64)).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(inner.name)).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("capacity"), &JsValue::from_f64(inner.capacity as f64)).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("description"), &JsValue::from_str(inner.description)).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("pickable"), &JsValue::from_bool(inner.pickable)).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("x"), &JsValue::from_f64(inner.x as f64)).unwrap();
                    Reflect::set(&obj, &JsValue::from_str("y"), &JsValue::from_f64(inner.y as f64)).unwrap();
                    let contents_arr = Array::new();
                    for &c in inner.contents.iter() {
                        contents_arr.push(&JsValue::from_f64(c as f64));
                    }
                    Reflect::set(&obj, &JsValue::from_str("contents"), &contents_arr).unwrap();
                    arr.push(&obj);
                }
            }
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
    /// Names of each room ID (0..rooms.len())
    #[wasm_bindgen]
    pub fn get_room_names(&self) -> Array {
        let arr = Array::new();
        for &name in self.sim.world.layout.room_names.iter() {
            arr.push(&JsValue::from_str(name));
        }
        arr
    }
    /// Check if the object with given ID is currently in its correct target location.
    #[wasm_bindgen]
    pub fn check_placement(&self, object_id: u32) -> bool {
        // delegate to core implementation
        if let Some(obj) = self.sim.world.objects.iter().find(|o| o.id as u32 == object_id) {
            return obj.check_placement(&self.sim.world);
        }
        false
    }
}
