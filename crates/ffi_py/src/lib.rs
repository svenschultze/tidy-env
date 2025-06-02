use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use tidy_core;

/// Python wrapper for GenOpts
#[pyclass]
#[derive(Clone)]
pub struct PyGenOpts {
    #[pyo3(get, set)]
    pub seed: u64,
    #[pyo3(get, set)]
    pub max_rooms: usize,
    #[pyo3(get, set)]
    pub width: usize,
    #[pyo3(get, set)]
    pub height: usize,
    #[pyo3(get, set)]
    pub max_objects: usize,
}

#[pymethods]
impl PyGenOpts {
    #[new]
    fn new(seed: u64, max_rooms: usize, width: usize, height: usize, max_objects: usize) -> Self {
        PyGenOpts {
            seed,
            max_rooms,
            width,
            height,
            max_objects,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "PyGenOpts(seed={}, max_rooms={}, width={}, height={}, max_objects={})",
            self.seed, self.max_rooms, self.width, self.height, self.max_objects
        )
    }
}

impl From<PyGenOpts> for tidy_core::GenOpts {
    fn from(opts: PyGenOpts) -> Self {
        tidy_core::GenOpts {
            seed: opts.seed,
            max_rooms: opts.max_rooms,
            width: opts.width,
            height: opts.height,
            max_objects: opts.max_objects,
        }
    }
}

/// Python wrapper for Object
#[pyclass]
#[derive(Clone)]
pub struct PyObject {
    #[pyo3(get)]
    pub id: usize,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub capacity: usize,
    #[pyo3(get)]
    pub pickable: bool,
    #[pyo3(get, set)]
    pub x: usize,
    #[pyo3(get, set)]
    pub y: usize,
    #[pyo3(get)]
    pub contents: Vec<usize>,
    #[pyo3(get)]
    pub description: String,
}

#[pymethods]
impl PyObject {
    fn __repr__(&self) -> String {
        format!(
            "PyObject(id={}, name='{}', x={}, y={}, pickable={}, capacity={})",
            self.id, self.name, self.x, self.y, self.pickable, self.capacity
        )
    }

    fn to_dict(&self) -> PyResult<PyObject> {
        Ok(PyObject {
            id: self.id,
            name: self.name.clone(),
            capacity: self.capacity,
            pickable: self.pickable,
            x: self.x,
            y: self.y,
            contents: self.contents.clone(),
            description: self.description.clone(),
        })
    }
}

impl From<&tidy_core::Object> for PyObject {
    fn from(obj: &tidy_core::Object) -> Self {
        PyObject {
            id: obj.id,
            name: obj.name.to_string(),
            capacity: obj.capacity,
            pickable: obj.pickable,
            x: obj.x,
            y: obj.y,
            contents: obj.contents.clone(),
            description: obj.description.to_string(),
        }
    }
}

/// Python wrapper for Layout
#[pyclass]
pub struct PyLayout {
    #[pyo3(get)]
    pub width: usize,
    #[pyo3(get)]
    pub height: usize,
    #[pyo3(get)]
    pub cells: Vec<i8>,
    #[pyo3(get)]
    pub room_names: Vec<String>,
}

#[pymethods]
impl PyLayout {
    fn __repr__(&self) -> String {
        format!(
            "PyLayout(width={}, height={}, rooms={})",
            self.width, self.height, self.room_names.len()
        )
    }

    fn get_cell(&self, x: usize, y: usize) -> PyResult<i8> {
        if x >= self.width || y >= self.height {
            return Err(PyValueError::new_err("Coordinates out of bounds"));
        }
        Ok(self.cells[y * self.width + x])
    }

    fn get_room_name(&self, room_id: usize) -> PyResult<String> {
        if room_id >= self.room_names.len() {
            return Err(PyValueError::new_err("Room ID out of bounds"));
        }
        Ok(self.room_names[room_id].clone())
    }
}

impl From<&tidy_core::Layout> for PyLayout {
    fn from(layout: &tidy_core::Layout) -> Self {
        PyLayout {
            width: layout.width,
            height: layout.height,
            cells: layout.cells.clone(),
            room_names: layout.room_names.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

/// Python wrapper for Simulator
#[pyclass]
pub struct PySimulator {
    sim: tidy_core::Simulator,
}

#[pymethods]
impl PySimulator {
    #[new]
    fn new(opts: PyGenOpts) -> PyResult<Self> {
        let rust_opts = tidy_core::GenOpts::from(opts);
        let world = tidy_core::generate(&rust_opts);
        
        // Auto-select first room cell for agent start
        let mut start_x = 0;
        let mut start_y = 0;
        for (i, &cell) in world.layout.cells.iter().enumerate() {
            if cell >= 0 {
                start_x = i % world.layout.width;
                start_y = i / world.layout.width;
                break;
            }
        }
        
        match tidy_core::Simulator::new(world, start_x, start_y) {
            Ok(sim) => Ok(PySimulator { sim }),
            Err(e) => Err(PyRuntimeError::new_err(e)),
        }
    }

    #[staticmethod]
    fn from_world_and_position(opts: PyGenOpts, start_x: usize, start_y: usize) -> PyResult<Self> {
        let rust_opts = tidy_core::GenOpts::from(opts);
        let world = tidy_core::generate(&rust_opts);
        
        match tidy_core::Simulator::new(world, start_x, start_y) {
            Ok(sim) => Ok(PySimulator { sim }),
            Err(e) => Err(PyRuntimeError::new_err(e)),
        }
    }

    #[getter]
    fn agent_x(&self) -> usize {
        self.sim.agent.x
    }

    #[getter]
    fn agent_y(&self) -> usize {
        self.sim.agent.y
    }

    fn move_up(&mut self) -> PyResult<()> {
        self.sim.up().map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn move_down(&mut self) -> PyResult<()> {
        self.sim.down().map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn move_left(&mut self) -> PyResult<()> {
        self.sim.left().map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn move_right(&mut self) -> PyResult<()> {
        self.sim.right().map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn interact(&mut self, dx: i32, dy: i32) -> PyResult<()> {
        self.sim
            .interact(dx as isize, dy as isize)
            .map_err(|e| PyRuntimeError::new_err(e))
    }

    fn open_door_up(&mut self) -> PyResult<()> {
        self.interact(0, -1)
    }

    fn open_door_down(&mut self) -> PyResult<()> {
        self.interact(0, 1)
    }

    fn open_door_left(&mut self) -> PyResult<()> {
        self.interact(-1, 0)
    }

    fn open_door_right(&mut self) -> PyResult<()> {
        self.interact(1, 0)
    }

    fn pick_up(&mut self) -> PyResult<()> {
        self.sim.pick_up().map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn drop(&mut self) -> PyResult<()> {
        self.sim.drop().map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn place_into(&mut self, target_id: usize) -> PyResult<()> {
        self.sim
            .place_into(target_id)
            .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
    }

    fn get_layout(&self) -> PyLayout {
        PyLayout::from(&self.sim.world.layout)
    }

    fn get_objects(&self) -> Vec<PyObject> {
        self.sim.world.objects.iter().map(PyObject::from).collect()
    }

    fn get_holding(&self) -> Option<PyObject> {
        self.sim.holding.as_ref().map(PyObject::from)
    }

    fn get_objects_at(&self, x: usize, y: usize) -> Vec<PyObject> {
        self.sim
            .world
            .objects
            .iter()
            .filter(|obj| obj.x == x && obj.y == y)
            .map(PyObject::from)
            .collect()
    }

    fn get_object_by_id(&self, id: usize) -> Option<PyObject> {
        self.sim
            .world
            .objects
            .iter()
            .find(|obj| obj.id == id)
            .map(PyObject::from)
    }

    fn check_placement(&self, object_id: usize) -> bool {
        if let Some(obj) = self.sim.world.objects.iter().find(|o| o.id == object_id) {
            obj.check_placement(&self.sim.world)
        } else {
            false
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "PySimulator(agent=({}, {}), objects={}, layout={}x{})",
            self.sim.agent.x,
            self.sim.agent.y,
            self.sim.world.objects.len(),
            self.sim.world.layout.width,
            self.sim.world.layout.height
        )
    }
}

/// Generate a world without creating a simulator
#[pyfunction]
fn generate_world(opts: PyGenOpts) -> (PyLayout, Vec<PyObject>) {
    let rust_opts = tidy_core::GenOpts::from(opts);
    let world = tidy_core::generate(&rust_opts);
    let layout = PyLayout::from(&world.layout);
    let objects = world.objects.iter().map(PyObject::from).collect();
    (layout, objects)
}

/// Constants module
#[pymodule]
fn constants(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("WALL", tidy_core::WALL)?;
    m.add("OUTSIDE", tidy_core::OUTSIDE)?;
    m.add("CLOSED_DOOR", tidy_core::CLOSED_DOOR)?;
    m.add("OPEN_DOOR", tidy_core::OPEN_DOOR)?;
    Ok(())
}

/// The main Python module
#[pymodule]
fn tidy_env_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGenOpts>()?;
    m.add_class::<PyObject>()?;
    m.add_class::<PyLayout>()?;
    m.add_class::<PySimulator>()?;
    m.add_function(wrap_pyfunction!(generate_world, m)?)?;
    
    // Create and add constants submodule
    let constants_module = PyModule::new(_py, "constants")?;
    constants(_py, constants_module)?;
    m.add_submodule(constants_module)?;
    
    Ok(())
}