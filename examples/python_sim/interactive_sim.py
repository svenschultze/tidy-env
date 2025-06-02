#!/usr/bin/env python3
"""
Interactive Apartment Simulator

A command-line interface for the tidy-env apartment simulation.
Provides movement, object interaction, and visualization capabilities.

Usage:
    python interactive_sim.py [--seed SEED] [--width WIDTH] [--height HEIGHT] [--rooms ROOMS] [--objects OBJECTS]

Commands:
    Movement: w/a/s/d (up/left/down/right)
    Doors: W/A/S/D (open door up/left/down/right)
    Actions: p (pick up), r (drop), i (interact with object)
    View: l (look around), m (show map), o (list objects), h (help)
    Quit: q (quit)
"""

import sys
import argparse
import os
from typing import Optional, List, Tuple

# Try to import tidy_env_py directly (if installed in venv)
try:
    import tidy_env_py
    from tidy_env_py import constants
except ImportError:
    # Fallback: Add the parent directory to Python path to import tidy_env_py
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'crates', 'ffi_py'))
    try:
        import tidy_env_py
        from tidy_env_py import constants
    except ImportError as e:
        print("Error: Could not import tidy_env_py.")
        print("Make sure you've built the Python bindings with:")
        print("  cd ../../crates/ffi_py")
        print("  maturin develop")
        print("Or activate the virtual environment if you have one set up.")
        print(f"Import error: {e}")
        sys.exit(1)


class InteractiveSimulator:
    def __init__(self, opts: tidy_env_py.PyGenOpts):
        """Initialize the interactive simulator."""
        self.sim = tidy_env_py.PySimulator(opts)
        self.layout = self.sim.get_layout()
        print(f"🏠 Welcome to the Apartment Simulator!")
        print(f"Generated apartment: {self.layout.width}x{self.layout.height} with {len(self.layout.room_names)} rooms")
        print(f"Total objects: {len(self.sim.get_objects())}")
        print(f"Agent starting position: ({self.sim.agent_x}, {self.sim.agent_y})")
        print("\nType 'h' for help or 'q' to quit.")
        
    def get_cell_char(self, x: int, y: int) -> str:
        """Get the character representation of a cell."""
        if x >= self.layout.width or y >= self.layout.height:
            return ' '
            
        cell_value = self.layout.get_cell(x, y)
        
        # Check if agent is at this position
        if x == self.sim.agent_x and y == self.sim.agent_y:
            return '@'
            
        # Check for objects at this position
        objects_here = self.sim.get_objects_at(x, y)
        if objects_here:
            # Show the first object's symbol
            obj = objects_here[0]
            if obj.pickable:
                return '.'  # Small pickable items
            else:
                return '#'  # Large objects/furniture
        
        # Show cell type
        if cell_value == constants.WALL:
            return '█'
        elif cell_value == constants.OUTSIDE:
            return ' '
        elif cell_value == constants.CLOSED_DOOR:
            return '+'
        elif cell_value == constants.OPEN_DOOR:
            return '-'
        else:
            # Room cell
            return '·'
    
    def show_map(self, radius: int = 5) -> None:
        """Display a map around the agent."""
        agent_x, agent_y = self.sim.agent_x, self.sim.agent_y
        
        print("\n" + "="*50)
        print("MAP (@ = you, · = floor, █ = wall, + = closed door, - = open door)")
        print("    . = small object, # = furniture")
        print("="*50)
        
        # Show a window around the agent
        start_x = max(0, agent_x - radius)
        end_x = min(self.layout.width, agent_x + radius + 1)
        start_y = max(0, agent_y - radius)
        end_y = min(self.layout.height, agent_y + radius + 1)
        
        # Print column numbers
        print("   ", end="")
        for x in range(start_x, end_x):
            print(f"{x%10}", end="")
        print()
        
        for y in range(start_y, end_y):
            print(f"{y:2} ", end="")
            for x in range(start_x, end_x):
                print(self.get_cell_char(x, y), end="")
            print(f" {y}")
        
        # Print column numbers again
        print("   ", end="")
        for x in range(start_x, end_x):
            print(f"{x%10}", end="")
        print()
    
    def look_around(self) -> None:
        """Look around the current position."""
        x, y = self.sim.agent_x, self.sim.agent_y
        print(f"\n👁  Looking around at ({x}, {y}):")
        
        # Get current room info
        cell_value = self.layout.get_cell(x, y)
        if cell_value >= 0:
            room_name = self.layout.get_room_name(cell_value)
            print(f"📍 You are in: {room_name}")
        
        # List objects at current location
        objects_here = self.sim.get_objects_at(x, y)
        if objects_here:
            print("🔍 Objects here:")
            for obj in objects_here:
                status = "📦" if obj.pickable else "🪑"
                contents_info = f" (contains {len(obj.contents)} items)" if obj.contents else ""
                print(f"  {status} {obj.name}: {obj.description}{contents_info}")
        else:
            print("🔍 No objects here.")
        
        # Show what you're holding
        holding = self.sim.get_holding()
        if holding:
            print(f"🤏 Holding: {holding.name}")
        else:
            print("🤏 Hands are empty.")
        
        # Check adjacent cells for doors or objects
        directions = [
            (0, -1, "north"),
            (0, 1, "south"), 
            (-1, 0, "west"),
            (1, 0, "east")
        ]
        
        adjacent_info = []
        for dx, dy, direction in directions:
            adj_x, adj_y = x + dx, y + dy
            if 0 <= adj_x < self.layout.width and 0 <= adj_y < self.layout.height:
                adj_cell = self.layout.get_cell(adj_x, adj_y)
                if adj_cell == constants.CLOSED_DOOR:
                    adjacent_info.append(f"🚪 Closed door to {direction}")
                elif adj_cell == constants.OPEN_DOOR:
                    adjacent_info.append(f"🚪 Open door to {direction}")
                
                adj_objects = self.sim.get_objects_at(adj_x, adj_y)
                if adj_objects:
                    obj_names = [obj.name for obj in adj_objects[:2]]
                    if len(adj_objects) > 2:
                        obj_names.append("...")
                    adjacent_info.append(f"🔍 {direction}: {', '.join(obj_names)}")
        
        if adjacent_info:
            print("🧭 Nearby:")
            for info in adjacent_info:
                print(f"  {info}")
    
    def list_all_objects(self) -> None:
        """List all objects in the world."""
        objects = self.sim.get_objects()
        print(f"\n📋 All Objects ({len(objects)} total):")
        
        # Group by room
        rooms = {}
        for obj in objects:
            cell_value = self.layout.get_cell(obj.x, obj.y)
            if cell_value >= 0:
                room_name = self.layout.get_room_name(cell_value)
            else:
                room_name = "Outside"
            
            if room_name not in rooms:
                rooms[room_name] = []
            rooms[room_name].append(obj)
        
        for room_name, room_objects in rooms.items():
            print(f"\n🏠 {room_name}:")
            for obj in room_objects:
                status = "📦" if obj.pickable else "🪑"
                contents_info = f" ({len(obj.contents)} items)" if obj.contents else ""
                print(f"  {status} {obj.name} at ({obj.x}, {obj.y}){contents_info}")
    
    def show_help(self) -> None:
        """Show help information."""
        print("\n" + "="*50)
        print("HELP - Interactive Apartment Simulator")
        print("="*50)
        print("MOVEMENT:")
        print("  w - Move up (north)")
        print("  s - Move down (south)")
        print("  a - Move left (west)")
        print("  d - Move right (east)")
        print()
        print("DOOR INTERACTION:")
        print("  W - Open/close door to the north")
        print("  S - Open/close door to the south")
        print("  A - Open/close door to the west")
        print("  D - Open/close door to the east")
        print()
        print("OBJECT INTERACTION:")
        print("  p - Pick up object at current location")
        print("  r - Drop held object")
        print("  i - Place held object into container")
        print()
        print("INFORMATION:")
        print("  l - Look around current location")
        print("  m - Show map")
        print("  o - List all objects")
        print("  h - Show this help")
        print()
        print("GAME:")
        print("  q - Quit")
        print("="*50)
    
    def handle_move(self, direction: str) -> bool:
        """Handle movement commands. Returns True if successful."""
        try:
            if direction == 'w':
                self.sim.move_up()
            elif direction == 's':
                self.sim.move_down()
            elif direction == 'a':
                self.sim.move_left()
            elif direction == 'd':
                self.sim.move_right()
            else:
                return False
            
            print(f"✅ Moved to ({self.sim.agent_x}, {self.sim.agent_y})")
            return True
        except RuntimeError as e:
            print(f"❌ Cannot move: {e}")
            return False
    
    def handle_door(self, direction: str) -> bool:
        """Handle door interaction commands."""
        direction_map = {
            'W': (0, -1, "north"),
            'S': (0, 1, "south"),
            'A': (-1, 0, "west"),
            'D': (1, 0, "east")
        }
        
        if direction not in direction_map:
            return False
        
        dx, dy, dir_name = direction_map[direction]
        
        try:
            self.sim.interact(dx, dy)
            print(f"🚪 Interacted with door to the {dir_name}")
            return True
        except RuntimeError as e:
            print(f"❌ Cannot interact with door: {e}")
            return False
    
    def handle_pickup(self) -> bool:
        """Handle pickup command."""
        objects_here = self.sim.get_objects_at(self.sim.agent_x, self.sim.agent_y)
        pickable_objects = [obj for obj in objects_here if obj.pickable]
        
        if not pickable_objects:
            print("❌ No pickable objects here.")
            return False
        
        try:
            self.sim.pick_up()
            holding = self.sim.get_holding()
            if holding:
                print(f"✅ Picked up: {holding.name}")
            return True
        except RuntimeError as e:
            print(f"❌ Cannot pick up: {e}")
            return False
    
    def handle_drop(self) -> bool:
        """Handle drop command."""
        holding = self.sim.get_holding()
        if not holding:
            print("❌ Not holding anything.")
            return False
        
        try:
            self.sim.drop()
            print(f"✅ Dropped: {holding.name}")
            return True
        except RuntimeError as e:
            print(f"❌ Cannot drop: {e}")
            return False
    
    def handle_place_into(self) -> bool:
        """Handle placing object into container."""
        holding = self.sim.get_holding()
        if not holding:
            print("❌ Not holding anything to place.")
            return False
        
        objects_here = self.sim.get_objects_at(self.sim.agent_x, self.sim.agent_y)
        containers = [obj for obj in objects_here if obj.capacity > 0 and obj.id != holding.id]
        
        if not containers:
            print("❌ No containers here.")
            return False
        
        if len(containers) == 1:
            target = containers[0]
        else:
            print("📦 Available containers:")
            for i, container in enumerate(containers):
                space = container.capacity - len(container.contents)
                print(f"  {i + 1}. {container.name} (space: {space})")
            
            try:
                choice = input("Choose container (number): ").strip()
                idx = int(choice) - 1
                if 0 <= idx < len(containers):
                    target = containers[idx]
                else:
                    print("❌ Invalid choice.")
                    return False
            except (ValueError, KeyboardInterrupt):
                print("❌ Invalid input.")
                return False
        
        try:
            self.sim.place_into(target.id)
            print(f"✅ Placed {holding.name} into {target.name}")
            return True
        except RuntimeError as e:
            print(f"❌ Cannot place into container: {e}")
            return False
    
    def run(self) -> None:
        """Run the interactive simulation loop."""
        self.show_map()
        self.look_around()
        
        while True:
            try:
                command = input("\n🎮 Command: ").strip()
                
                if not command:
                    continue
                
                if command == 'q':
                    print("👋 Goodbye!")
                    break
                elif command == 'h':
                    self.show_help()
                elif command == 'l':
                    self.look_around()
                elif command == 'm':
                    self.show_map()
                elif command == 'o':
                    self.list_all_objects()
                elif command in ['w', 'a', 's', 'd']:
                    self.handle_move(command)
                elif command in ['W', 'A', 'S', 'D']:
                    self.handle_door(command)
                elif command == 'p':
                    self.handle_pickup()
                elif command == 'r':
                    self.handle_drop()
                elif command == 'i':
                    self.handle_place_into()
                else:
                    print("❌ Unknown command. Type 'h' for help.")
                    
            except KeyboardInterrupt:
                print("\n👋 Goodbye!")
                break
            except EOFError:
                print("\n👋 Goodbye!")
                break


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Interactive Apartment Simulator")
    parser.add_argument("--seed", type=int, default=42, 
                       help="Random seed for generation (default: 42)")
    parser.add_argument("--width", type=int, default=20,
                       help="Apartment width (default: 20)")
    parser.add_argument("--height", type=int, default=20,
                       help="Apartment height (default: 20)")
    parser.add_argument("--rooms", type=int, default=5,
                       help="Maximum number of rooms (default: 5)")
    parser.add_argument("--objects", type=int, default=15,
                       help="Maximum number of objects (default: 15)")
    return parser.parse_args()


def main():
    """Main entry point."""
    args = parse_args()
    
    # Create generation options
    opts = tidy_env_py.PyGenOpts(
        seed=args.seed,
        max_rooms=args.rooms,
        width=args.width,
        height=args.height,
        max_objects=args.objects
    )
    
    # Create and run the interactive simulator
    sim = InteractiveSimulator(opts)
    sim.run()


if __name__ == "__main__":
    main()