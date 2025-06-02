# Python Interactive Simulator Examples

This directory contains interactive Python environments for the tidy-env apartment simulation.

## Prerequisites & Setup

### Option 1: Using the Virtual Environment (Recommended)

The project includes a pre-configured virtual environment with all dependencies installed:

1. **Activate the environment**:
   ```bash
   source activate.sh
   ```

2. **Run the simulators**:
   ```bash
   python interactive_sim.py --seed 42
   # or
   python visual_sim.py --seed 42
   ```

3. **Deactivate when done**:
   ```bash
   deactivate
   ```

### Option 2: Manual Setup

If you prefer to set up your own environment:

1. **Build the Python bindings**:
   ```bash
   cd ../../crates/ffi_py
   maturin develop
   ```

2. **Install optional dependencies**:
   ```bash
   pip install colorama  # For colored output in visual_sim.py
   ```

## Available Simulators

### 1. Basic Interactive Simulator (`interactive_sim.py`)

A command-line text interface with comprehensive features:

```bash
python interactive_sim.py --seed 42 --width 20 --height 20 --rooms 5 --objects 15
```

**Features:**
- Text-based map visualization
- Movement with WASD keys
- Door interaction with Shift+WASD
- Object pickup, drop, and container placement
- Room exploration and object inspection
- Full apartment overview

**Commands:**
- `w/a/s/d` - Move up/left/down/right
- `W/A/S/D` - Open/close doors
- `p` - Pick up objects
- `r` - Drop objects
- `i` - Place objects into containers
- `l` - Look around (detailed view)
- `m` - Show full map
- `o` - List all objects
- `h` - Help
- `q` - Quit

### 2. Visual Simulator (`visual_sim.py`)

A colorful, immersive ASCII art interface:

```bash
python visual_sim.py --seed 42 --width 25 --height 20 --rooms 6 --objects 20
```

**Features:**
- Colored ASCII art visualization
- Real-time status panel
- Local view around agent
- Step counter and progress tracking
- Interactive container selection
- Full apartment map view

**Visual Elements:**
- `@` (yellow) - Your character
- `·` (cyan) - Floor tiles
- `█` (white) - Walls
- `+` (red) - Closed doors
- `-` (green) - Open doors
- `.` (magenta) - Small pickable objects
- `█` (blue) - Large furniture/containers

## Command Line Options

Both simulators support the same command line arguments:

- `--seed SEED` - Random seed for generation (default: 42)
- `--width WIDTH` - Apartment width (default: 20/25)
- `--height HEIGHT` - Apartment height (default: 20)
- `--rooms ROOMS` - Maximum number of rooms (default: 5/6)
- `--objects OBJECTS` - Maximum number of objects (default: 15/20)

## Quick Start

```bash
# 1. Activate the virtual environment
source activate.sh

# 2. Run a simulator
python interactive_sim.py

# 3. Try different configurations
python visual_sim.py --width 30 --height 25 --rooms 8 --objects 25

# 4. Use a specific seed for reproducible layouts
python interactive_sim.py --seed 12345

# 5. Create a small apartment for testing
python visual_sim.py --width 15 --height 15 --rooms 3 --objects 8
```

## Gameplay Tips

1. **Exploration**: Start by using `m` to see the full map and `l` to examine your surroundings
2. **Movement**: Use doors (capital WASD) to move between rooms
3. **Objects**: Pick up small items with `p` and place them in containers with `i`
4. **Organization**: Try organizing objects by type or room
5. **Discovery**: Each apartment is randomly generated - explore to find all the rooms and objects!

## Troubleshooting

**Virtual Environment Issues**: If the activation script doesn't work, try:
```bash
source ../../venv/bin/activate
```

**Import Error**: Make sure the Python bindings are built:
```bash
cd ../../crates/ffi_py
maturin develop
```

**No Colors**: The virtual environment includes colorama, but if you see no colors:
```bash
pip install colorama
```

**Permission Issues**: Make sure the activation script is executable:
```bash
chmod +x activate.sh
```

## Development

The simulators automatically try to import from the installed package first, then fall back to path-based imports for development.

To modify or extend these simulators:

1. The core simulation logic is in the Rust crate (`../../crates/core`)
2. Python bindings are in `../../crates/ffi_py/src/lib.rs`
3. After making changes to Rust code, rebuild with `maturin develop`
4. Python changes take effect immediately

Feel free to create your own variations or add new features!