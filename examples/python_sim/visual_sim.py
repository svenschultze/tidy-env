#!/usr/bin/env python3
"""
Visual Apartment Simulator

A visual interface for the tidy-env apartment simulation using colored ASCII art.
Provides a more immersive experience with colors and better visualization.

Requirements: colorama (install with: pip install colorama)

Usage:
    python visual_sim.py [--seed SEED] [--width WIDTH] [--height HEIGHT] [--rooms ROOMS] [--objects OBJECTS]
"""

import sys
import argparse
import os
import time
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

try:
    from colorama import init, Fore, Back, Style
    init()  # Initialize colorama for Windows compatibility
    COLORS_AVAILABLE = True
except ImportError:
    print("Warning: colorama not installed. Install with 'pip install colorama' for colored output.")
    COLORS_AVAILABLE = False
    # Fallback color definitions
    class Fore:
        RED = GREEN = YELLOW = BLUE = MAGENTA = CYAN = WHITE = RESET = ""
    class Back:
        BLACK = RED = GREEN = YELLOW = BLUE = MAGENTA = CYAN = WHITE = RESET = ""
    class Style:
        BRIGHT = DIM = RESET_ALL = ""


class VisualSimulator:
    def __init__(self, opts: tidy_env_py.PyGenOpts):
        """Initialize the visual simulator."""
        self.sim = tidy_env_py.PySimulator(opts)
        self.layout = self.sim.get_layout()
        self.last_action = ""
        self.step_count = 0
        
        # Color scheme
        self.colors = {
            'wall': Fore.WHITE + Back.BLACK,
            'floor': Fore.CYAN + '·',
            'agent': Fore.YELLOW + Style.BRIGHT + '@',
            'door_closed': Fore.RED + '+',
            'door_open': Fore.GREEN + '-',
            'object_small': Fore.MAGENTA + '.',
            'object_large': Fore.BLUE + '█',
            'outside': ' ',
            'reset': Style.RESET_ALL
        }
        
        self.clear_screen()
        self.show_welcome()
    
    def clear_screen(self):
        """Clear the terminal screen."""
        os.system('cls' if os.name == 'nt' else 'clear')
    
    def show_welcome(self):
        """Show welcome message."""
        print(f"{Fore.GREEN}{'='*60}{Style.RESET_ALL}")
        print(f"{Fore.GREEN}🏠 VISUAL APARTMENT SIMULATOR 🏠{Style.RESET_ALL}")
        print(f"{Fore.GREEN}{'='*60}{Style.RESET_ALL}")
        print(f"Generated apartment: {self.layout.width}x{self.layout.height} with {len(self.layout.room_names)} rooms")
        print(f"Total objects: {len(self.sim.get_objects())}")
        print(f"Agent starting at: ({self.sim.agent_x}, {self.sim.agent_y})")
        print(f"\n{Fore.YELLOW}Controls:{Style.RESET_ALL}")
        print("WASD = Move, SHIFT+WASD = Doors, P = Pick up, R = Drop")
        print("I = Place into container, L = Look, O = Objects, H = Help, Q = Quit")
        input(f"\n{Fore.GREEN}Press Enter to start...{Style.RESET_ALL}")
    
    def get_cell_display(self, x: int, y: int) -> str:
        """Get the colored character representation of a cell."""
        if x >= self.layout.width or y >= self.layout.height:
            return self.colors['outside']
        
        cell_value = self.layout.get_cell(x, y)
        
        # Check if agent is at this position
        if x == self.sim.agent_x and y == self.sim.agent_y:
            return self.colors['agent'] + self.colors['reset']
        
        # Check for objects at this position
        objects_here = self.sim.get_objects_at(x, y)
        if objects_here:
            obj = objects_here[0]
            if obj.pickable:
                return self.colors['object_small'] + self.colors['reset']
            else:
                return self.colors['object_large'] + self.colors['reset']
        
        # Show cell type
        if cell_value == constants.WALL:
            return self.colors['wall'] + '█' + self.colors['reset']
        elif cell_value == constants.OUTSIDE:
            return self.colors['outside']
        elif cell_value == constants.CLOSED_DOOR:
            return self.colors['door_closed'] + self.colors['reset']
        elif cell_value == constants.OPEN_DOOR:
            return self.colors['door_open'] + self.colors['reset']
        else:
            # Room cell
            return self.colors['floor'] + self.colors['reset']
    
    def draw_full_map(self):
        """Draw the complete apartment map."""
        print(f"\n{Fore.CYAN}Full Apartment Map:{Style.RESET_ALL}")
        print("   ", end="")
        for x in range(min(self.layout.width, 30)):  # Limit width for readability
            print(f"{x%10}", end="")
        print()
        
        for y in range(min(self.layout.height, 20)):  # Limit height for readability
            print(f"{y:2} ", end="")
            for x in range(min(self.layout.width, 30)):
                print(self.get_cell_display(x, y), end="")
            print(f" {y}")
    
    def draw_local_view(self, radius: int = 7):
        """Draw a local view around the agent."""
        agent_x, agent_y = self.sim.agent_x, self.sim.agent_y
        
        start_x = max(0, agent_x - radius)
        end_x = min(self.layout.width, agent_x + radius + 1)
        start_y = max(0, agent_y - radius)
        end_y = min(self.layout.height, agent_y + radius + 1)
        
        print(f"\n{Fore.YELLOW}Local View (Agent at {agent_x}, {agent_y}):{Style.RESET_ALL}")
        
        # Print column numbers
        print("   ", end="")
        for x in range(start_x, end_x):
            print(f"{x%10}", end="")
        print()
        
        for y in range(start_y, end_y):
            print(f"{y:2} ", end="")
            for x in range(start_x, end_x):
                print(self.get_cell_display(x, y), end="")
            print(f" {y}")
    
    def show_status_panel(self):
        """Show the current status panel."""
        print(f"\n{Fore.GREEN}{'='*50}{Style.RESET_ALL}")
        print(f"{Fore.GREEN}STATUS{Style.RESET_ALL}")
        print(f"{Fore.GREEN}{'='*50}{Style.RESET_ALL}")
        
        # Agent info
        x, y = self.sim.agent_x, self.sim.agent_y
        cell_value = self.layout.get_cell(x, y)
        room_name = "Outside"
        if cell_value >= 0:
            room_name = self.layout.get_room_name(cell_value)
        
        print(f"{Fore.YELLOW}Location:{Style.RESET_ALL} ({x}, {y}) in {room_name}")
        print(f"{Fore.YELLOW}Steps:{Style.RESET_ALL} {self.step_count}")
        
        # What you're holding
        holding = self.sim.get_holding()
        if holding:
            print(f"{Fore.YELLOW}Holding:{Style.RESET_ALL} {Fore.MAGENTA}{holding.name}{Style.RESET_ALL}")
        else:
            print(f"{Fore.YELLOW}Holding:{Style.RESET_ALL} Nothing")
        
        # Objects at current location
        objects_here = self.sim.get_objects_at(x, y)
        if objects_here:
            print(f"{Fore.YELLOW}Objects here:{Style.RESET_ALL}")
            for obj in objects_here:
                status = f"{Fore.MAGENTA}📦" if obj.pickable else f"{Fore.BLUE}🪑"
                print(f"  {status} {obj.name}{Style.RESET_ALL}")
        
        # Last action
        if self.last_action:
            print(f"{Fore.YELLOW}Last action:{Style.RESET_ALL} {self.last_action}")
        
        # Legend
        print(f"\n{Fore.CYAN}Legend:{Style.RESET_ALL}")
        print(f"  {self.colors['agent']}@{Style.RESET_ALL} = You")
        print(f"  {self.colors['floor']}{Style.RESET_ALL} = Floor")
        print(f"  {self.colors['wall']}█{Style.RESET_ALL} = Wall")
        print(f"  {self.colors['door_closed']}{Style.RESET_ALL} = Closed door")
        print(f"  {self.colors['door_open']}{Style.RESET_ALL} = Open door")
        print(f"  {self.colors['object_small']}{Style.RESET_ALL} = Small object")
        print(f"  {self.colors['object_large']}{Style.RESET_ALL} = Furniture")
    
    def show_inventory_and_nearby(self):
        """Show detailed inventory and nearby objects."""
        print(f"\n{Fore.CYAN}DETAILED VIEW{Style.RESET_ALL}")
        print("="*30)
        
        # Current room objects
        x, y = self.sim.agent_x, self.sim.agent_y
        objects_here = self.sim.get_objects_at(x, y)
        
        if objects_here:
            print(f"\n{Fore.YELLOW}Objects at your location:{Style.RESET_ALL}")
            for i, obj in enumerate(objects_here):
                status = "📦 Pickable" if obj.pickable else "🪑 Furniture"
                capacity_info = ""
                if obj.capacity > 0:
                    space = obj.capacity - len(obj.contents)
                    capacity_info = f" (Container: {space} spaces free)"
                print(f"  {i+1}. {Fore.MAGENTA}{obj.name}{Style.RESET_ALL}: {obj.description}")
                print(f"      {status}{capacity_info}")
        
        # Nearby areas
        directions = [(0, -1, "North"), (0, 1, "South"), (-1, 0, "West"), (1, 0, "East")]
        for dx, dy, direction in directions:
            adj_x, adj_y = x + dx, y + dy
            if 0 <= adj_x < self.layout.width and 0 <= adj_y < self.layout.height:
                adj_cell = self.layout.get_cell(adj_x, adj_y)
                adj_objects = self.sim.get_objects_at(adj_x, adj_y)
                
                info_parts = []
                if adj_cell == constants.CLOSED_DOOR:
                    info_parts.append(f"{Fore.RED}Closed door{Style.RESET_ALL}")
                elif adj_cell == constants.OPEN_DOOR:
                    info_parts.append(f"{Fore.GREEN}Open door{Style.RESET_ALL}")
                elif adj_cell == constants.WALL:
                    info_parts.append("Wall")
                elif adj_cell >= 0:
                    room_name = self.layout.get_room_name(adj_cell)
                    info_parts.append(f"Room: {room_name}")
                
                if adj_objects:
                    obj_names = [obj.name for obj in adj_objects[:2]]
                    if len(adj_objects) > 2:
                        obj_names.append("...")
                    info_parts.append(f"Objects: {', '.join(obj_names)}")
                
                if info_parts:
                    print(f"{Fore.CYAN}{direction}:{Style.RESET_ALL} {', '.join(info_parts)}")
    
    def refresh_display(self):
        """Refresh the entire display."""
        self.clear_screen()
        self.draw_local_view()
        self.show_status_panel()
    
    def handle_command(self, command: str) -> bool:
        """Handle a single command. Returns False if should quit."""
        command = command.strip()
        self.last_action = ""
        
        try:
            if command == 'q':
                return False
            elif command == 'w':
                self.sim.move_up()
                self.last_action = f"{Fore.GREEN}Moved up{Style.RESET_ALL}"
                self.step_count += 1
            elif command == 's':
                self.sim.move_down()
                self.last_action = f"{Fore.GREEN}Moved down{Style.RESET_ALL}"
                self.step_count += 1
            elif command == 'a':
                self.sim.move_left()
                self.last_action = f"{Fore.GREEN}Moved left{Style.RESET_ALL}"
                self.step_count += 1
            elif command == 'd':
                self.sim.move_right()
                self.last_action = f"{Fore.GREEN}Moved right{Style.RESET_ALL}"
                self.step_count += 1
            elif command == 'W':
                self.sim.interact(0, -1)
                self.last_action = f"{Fore.YELLOW}Interacted with door (north){Style.RESET_ALL}"
            elif command == 'S':
                self.sim.interact(0, 1)
                self.last_action = f"{Fore.YELLOW}Interacted with door (south){Style.RESET_ALL}"
            elif command == 'A':
                self.sim.interact(-1, 0)
                self.last_action = f"{Fore.YELLOW}Interacted with door (west){Style.RESET_ALL}"
            elif command == 'D':
                self.sim.interact(1, 0)
                self.last_action = f"{Fore.YELLOW}Interacted with door (east){Style.RESET_ALL}"
            elif command == 'p':
                self.sim.pick_up()
                holding = self.sim.get_holding()
                if holding:
                    self.last_action = f"{Fore.MAGENTA}Picked up: {holding.name}{Style.RESET_ALL}"
            elif command == 'r':
                holding = self.sim.get_holding()
                if holding:
                    self.sim.drop()
                    self.last_action = f"{Fore.MAGENTA}Dropped: {holding.name}{Style.RESET_ALL}"
                else:
                    self.last_action = f"{Fore.RED}Not holding anything{Style.RESET_ALL}"
            elif command == 'i':
                self.handle_place_into()
            elif command == 'l':
                self.clear_screen()
                self.show_inventory_and_nearby()
                input(f"\n{Fore.GREEN}Press Enter to continue...{Style.RESET_ALL}")
            elif command == 'o':
                self.show_all_objects()
            elif command == 'm':
                self.clear_screen()
                self.draw_full_map()
                input(f"\n{Fore.GREEN}Press Enter to continue...{Style.RESET_ALL}")
            elif command == 'h':
                self.show_help()
            else:
                self.last_action = f"{Fore.RED}Unknown command: {command}{Style.RESET_ALL}"
        
        except RuntimeError as e:
            self.last_action = f"{Fore.RED}Error: {e}{Style.RESET_ALL}"
        
        return True
    
    def handle_place_into(self):
        """Handle placing object into container with visual selection."""
        holding = self.sim.get_holding()
        if not holding:
            self.last_action = f"{Fore.RED}Not holding anything{Style.RESET_ALL}"
            return
        
        objects_here = self.sim.get_objects_at(self.sim.agent_x, self.sim.agent_y)
        containers = [obj for obj in objects_here if obj.capacity > 0 and obj.id != holding.id]
        
        if not containers:
            self.last_action = f"{Fore.RED}No containers here{Style.RESET_ALL}"
            return
        
        if len(containers) == 1:
            target = containers[0]
        else:
            self.clear_screen()
            print(f"{Fore.CYAN}Choose a container for {holding.name}:{Style.RESET_ALL}")
            for i, container in enumerate(containers):
                space = container.capacity - len(container.contents)
                print(f"  {i + 1}. {Fore.BLUE}{container.name}{Style.RESET_ALL} (space: {space})")
            
            try:
                choice = input(f"\n{Fore.YELLOW}Choose container (number): {Style.RESET_ALL}").strip()
                idx = int(choice) - 1
                if 0 <= idx < len(containers):
                    target = containers[idx]
                else:
                    self.last_action = f"{Fore.RED}Invalid choice{Style.RESET_ALL}"
                    return
            except (ValueError, KeyboardInterrupt):
                self.last_action = f"{Fore.RED}Cancelled{Style.RESET_ALL}"
                return
        
        try:
            self.sim.place_into(target.id)
            self.last_action = f"{Fore.GREEN}Placed {holding.name} into {target.name}{Style.RESET_ALL}"
        except RuntimeError as e:
            self.last_action = f"{Fore.RED}Cannot place: {e}{Style.RESET_ALL}"
    
    def show_all_objects(self):
        """Show all objects grouped by room."""
        self.clear_screen()
        objects = self.sim.get_objects()
        print(f"{Fore.CYAN}ALL OBJECTS IN APARTMENT ({len(objects)} total){Style.RESET_ALL}")
        print("="*50)
        
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
            print(f"\n{Fore.YELLOW}🏠 {room_name}:{Style.RESET_ALL}")
            for obj in room_objects:
                status = f"{Fore.MAGENTA}📦" if obj.pickable else f"{Fore.BLUE}🪑"
                contents_info = f" ({len(obj.contents)} items)" if obj.contents else ""
                print(f"  {status} {obj.name}{Style.RESET_ALL} at ({obj.x}, {obj.y}){contents_info}")
        
        input(f"\n{Fore.GREEN}Press Enter to continue...{Style.RESET_ALL}")
    
    def show_help(self):
        """Show help screen."""
        self.clear_screen()
        print(f"{Fore.GREEN}{'='*50}{Style.RESET_ALL}")
        print(f"{Fore.GREEN}HELP - Visual Apartment Simulator{Style.RESET_ALL}")
        print(f"{Fore.GREEN}{'='*50}{Style.RESET_ALL}")
        print(f"\n{Fore.YELLOW}MOVEMENT:{Style.RESET_ALL}")
        print("  w, a, s, d - Move up, left, down, right")
        print(f"\n{Fore.YELLOW}DOOR INTERACTION:{Style.RESET_ALL}")
        print("  W, A, S, D - Open/close door up, left, down, right")
        print(f"\n{Fore.YELLOW}OBJECT INTERACTION:{Style.RESET_ALL}")
        print("  p - Pick up object at current location")
        print("  r - Drop held object")
        print("  i - Place held object into container")
        print(f"\n{Fore.YELLOW}INFORMATION:{Style.RESET_ALL}")
        print("  l - Look around (detailed view)")
        print("  m - Show full map")
        print("  o - List all objects")
        print("  h - Show this help")
        print(f"\n{Fore.YELLOW}GAME:{Style.RESET_ALL}")
        print("  q - Quit")
        print(f"\n{Fore.CYAN}The goal is to explore the apartment and interact with objects!")
        print(f"Try picking up items and placing them in containers.{Style.RESET_ALL}")
        
        input(f"\n{Fore.GREEN}Press Enter to continue...{Style.RESET_ALL}")
    
    def run(self):
        """Run the visual simulation loop."""
        self.refresh_display()
        
        while True:
            try:
                print(f"\n{Fore.GREEN}Command (h for help): {Style.RESET_ALL}", end="")
                command = input().strip()
                
                if not command:
                    continue
                
                if not self.handle_command(command):
                    break
                
                self.refresh_display()
                
            except KeyboardInterrupt:
                print(f"\n{Fore.YELLOW}Goodbye!{Style.RESET_ALL}")
                break
            except EOFError:
                print(f"\n{Fore.YELLOW}Goodbye!{Style.RESET_ALL}")
                break


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Visual Apartment Simulator")
    parser.add_argument("--seed", type=int, default=42, 
                       help="Random seed for generation (default: 42)")
    parser.add_argument("--width", type=int, default=25,
                       help="Apartment width (default: 25)")
    parser.add_argument("--height", type=int, default=20,
                       help="Apartment height (default: 20)")
    parser.add_argument("--rooms", type=int, default=6,
                       help="Maximum number of rooms (default: 6)")
    parser.add_argument("--objects", type=int, default=20,
                       help="Maximum number of objects (default: 20)")
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
    
    # Create and run the visual simulator
    sim = VisualSimulator(opts)
    sim.run()


if __name__ == "__main__":
    main()