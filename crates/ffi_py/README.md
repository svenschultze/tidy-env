# Python Bindings for Tidy-Env

Python bindings for the tidy-env apartment simulator using PyO3.

## Installation

You need to have Rust and Python installed. Then:

```bash
# Install maturin if you haven't already
pip install maturin

# Build and install the Python package
cd crates/ffi_py
maturin develop
```

For release builds:
```bash
maturin develop --release
```

## Usage

```python
import tidy_env_py

# Create generation options
opts = tidy_env_py.PyGenOpts(
    seed=42,
    max_rooms=5,
    width=20,
    height=20,
    max_objects=15
)

# Create a simulator
sim = tidy_env_py.PySimulator(opts)

# Get the layout
layout = sim.get_layout()
print(f"Layout: {layout.width}x{layout.height} with {len(layout.room_names)} rooms")

# Get agent position
print(f"Agent at: ({sim.agent_x}, {sim.agent_y})")

# Move the agent
try:
    sim.move_right()
    print(f"Moved to: ({sim.agent_x}, {sim.agent_y})")
except RuntimeError as e:
    print(f"Cannot move: {e}")

# Get objects in the world
objects = sim.get_objects()
print(f"Found {len(objects)} objects:")
for obj in objects[:5]:  # Show first 5
    print(f"  {obj}")

# Try to pick up an object at current location
objects_here = sim.get_objects_at(sim.agent_x, sim.agent_y)
if objects_here:
    pickable = [obj for obj in objects_here if obj.pickable]
    if pickable:
        try:
            sim.pick_up()
            holding = sim.get_holding()
            print(f"Picked up: {holding}")
        except RuntimeError as e:
            print(f"Cannot pick up: {e}")

# Access constants
from tidy_env_py import constants
print(f"Wall value: {constants.WALL}")
print(f"Outside value: {constants.OUTSIDE}")
```

## API Reference

### PyGenOpts
- `seed: int` - Random seed for generation
- `max_rooms: int` - Maximum number of rooms
- `width: int` - Layout width
- `height: int` - Layout height  
- `max_objects: int` - Maximum number of objects

### PySimulator
- `agent_x`, `agent_y` - Agent position (read-only)
- `move_up()`, `move_down()`, `move_left()`, `move_right()` - Move agent
- `pick_up()`, `drop()` - Pick up/drop objects
- `interact(dx, dy)` - Interact with doors/objects at relative position
- `get_layout()` - Get the layout
- `get_objects()` - Get all objects
- `get_holding()` - Get currently held object
- `get_objects_at(x, y)` - Get objects at specific position

### PyLayout
- `width`, `height` - Layout dimensions
- `cells` - Flat array of cell values
- `room_names` - List of room names
- `get_cell(x, y)` - Get cell value at position
- `get_room_name(room_id)` - Get name of room by ID

### PyObject
- `id`, `name`, `description` - Object identification
- `x`, `y` - Object position
- `pickable` - Whether object can be picked up
- `capacity` - Container capacity
- `contents` - List of contained object IDs