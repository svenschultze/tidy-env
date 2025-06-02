#!/usr/bin/env python3
"""
Example usage of the tidy_env_py Python bindings.
Run this after building the Python module with: maturin develop
"""

import tidy_env_py

def main():
    # Create generation options
    opts = tidy_env_py.PyGenOpts(
        seed=42,
        max_rooms=5,
        width=20,
        height=20,
        max_objects=15
    )
    print(f"Generation options: {opts}")

    # Create a simulator
    sim = tidy_env_py.PySimulator(opts)
    print(f"Created simulator: {sim}")

    # Get the layout
    layout = sim.get_layout()
    print(f"Layout: {layout.width}x{layout.height} with {len(layout.room_names)} rooms")
    print(f"Room names: {layout.room_names}")

    # Get agent position
    print(f"Agent at: ({sim.agent_x}, {sim.agent_y})")

    # Get objects in the world
    objects = sim.get_objects()
    print(f"Found {len(objects)} objects:")
    for obj in objects[:5]:  # Show first 5
        print(f"  {obj}")

    # Try to move the agent
    try:
        sim.move_right()
        print(f"Moved right to: ({sim.agent_x}, {sim.agent_y})")
    except RuntimeError as e:
        print(f"Cannot move right: {e}")

    try:
        sim.move_down()
        print(f"Moved down to: ({sim.agent_x}, {sim.agent_y})")
    except RuntimeError as e:
        print(f"Cannot move down: {e}")

    # Check for objects at current location
    objects_here = sim.get_objects_at(sim.agent_x, sim.agent_y)
    if objects_here:
        print(f"Objects at current location ({sim.agent_x}, {sim.agent_y}):")
        for obj in objects_here:
            print(f"  {obj}")
            
        # Try to pick up a pickable object
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
    print(f"Constants - Wall: {constants.WALL}, Outside: {constants.OUTSIDE}")
    print(f"Constants - Closed door: {constants.CLOSED_DOOR}, Open door: {constants.OPEN_DOOR}")

    # Generate a world without simulator
    layout, objects = tidy_env_py.generate_world(opts)
    print(f"Generated standalone world: {layout.width}x{layout.height} with {len(objects)} objects")

if __name__ == "__main__":
    main()