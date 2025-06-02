#!/usr/bin/env python3
"""
Navigation Challenge Demo

An interactive demo for testing pathfinding and path validation.
This script creates navigation challenges and lets you test different paths.

Usage:
    python navigation_demo.py [--seed SEED] [--width WIDTH] [--height HEIGHT]
"""

import sys
import argparse
import os
from typing import Optional

# Try to import tidy_env_py
try:
    import tidy_env_py
    from tidy_env_py import constants
except ImportError:
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'crates', 'ffi_py'))
    try:
        import tidy_env_py
        from tidy_env_py import constants
    except ImportError as e:
        print("Error: Could not import tidy_env_py.")
        print("Make sure you've built the Python bindings or activated the virtual environment.")
        print(f"Import error: {e}")
        sys.exit(1)

from pathfinding_validator import PathfindingValidator, demo_pathfinding


class NavigationDemo:
    def __init__(self, opts: tidy_env_py.PyGenOpts):
        """Initialize the navigation demo."""
        self.sim = tidy_env_py.PySimulator(opts)
        self.validator = PathfindingValidator(self.sim)
        self.layout = self.sim.get_layout()
        
        print(f"🏠 Navigation Challenge Demo")
        print(f"Generated apartment: {self.layout.width}x{self.layout.height}")
        print(f"Objects: {len(self.sim.get_objects())}")
        print(f"Agent at: ({self.sim.agent_x}, {self.sim.agent_y})")
        
    def show_map_with_target(self, target_pos: Optional[tuple] = None):
        """Show a simple map with optional target marked."""
        print("\n" + "="*60)
        print("MAP (@ = agent, T = target, · = floor, █ = wall, + = door)")
        print("="*60)
        
        # Show a reasonable view of the apartment
        max_display_width = min(60, self.layout.width)
        max_display_height = min(20, self.layout.height)
        
        for y in range(max_display_height):
            for x in range(max_display_width):
                if (x, y) == (self.sim.agent_x, self.sim.agent_y):
                    print('@', end='')
                elif target_pos and (x, y) == target_pos:
                    print('T', end='')
                else:
                    cell_value = self.layout.get_cell(x, y)
                    if cell_value == constants.WALL:
                        print('█', end='')
                    elif cell_value == constants.OUTSIDE:
                        print(' ', end='')
                    elif cell_value == constants.CLOSED_DOOR:
                        print('+', end='')
                    elif cell_value == constants.OPEN_DOOR:
                        print('-', end='')
                    else:
                        # Check for objects
                        objects_here = self.sim.get_objects_at(x, y)
                        if objects_here:
                            print('·', end='')  # Object marker
                        else:
                            print('·', end='')  # Floor
            print()
        
        if self.layout.width > max_display_width or self.layout.height > max_display_height:
            print(f"(Map truncated to {max_display_width}x{max_display_height})")
    
    def interactive_challenges(self):
        """Run interactive navigation challenges."""
        challenge_num = 1
        
        while True:
            print(f"\n{'='*60}")
            print(f"🎯 NAVIGATION CHALLENGE #{challenge_num}")
            print(f"{'='*60}")
            
            # Generate a challenge
            challenge = self.validator.create_navigation_challenge(seed=challenge_num * 42)
            if challenge is None:
                print("❌ Could not generate a navigation challenge.")
                print("This might happen if there are no objects or no valid paths.")
                break
            
            print(f"📍 {challenge.description}")
            print(f"🎯 Target object: {challenge.target_object_name}")
            print(f"🏁 Optimal solution: {challenge.optimal_path_arrows} ({challenge.optimal_path_length} steps)")
            
            # Show the map
            self.show_map_with_target(challenge.target)
            
            print(f"\n📝 Enter your path using arrow symbols (↑→↓←):")
            print(f"   Example: ↑→→↓ (up, right, right, down)")
            print(f"   Type 'hint' for the optimal path")
            print(f"   Type 'skip' to skip this challenge")
            print(f"   Type 'quit' to exit")
            
            while True:
                try:
                    user_input = input(f"\n🎮 Your path: ").strip()
                    
                    if user_input.lower() == 'quit':
                        print("👋 Thanks for playing!")
                        return
                    elif user_input.lower() == 'skip':
                        break
                    elif user_input.lower() == 'hint':
                        print(f"💡 Hint - Optimal path: {challenge.optimal_path_arrows}")
                        continue
                    elif not user_input:
                        print("❌ Please enter a path or command.")
                        continue
                    
                    # Validate the path
                    result = self.validator.validate_path(
                        challenge.start, 
                        challenge.target, 
                        user_input
                    )
                    
                    # Show results
                    analysis = self.validator.analyze_path_efficiency(result)
                    print(f"\n📊 Result: {analysis}")
                    
                    if result.reached_target:
                        print(f"🎉 Congratulations! You reached the target!")
                        if result.efficiency and result.efficiency >= 1.0:
                            print(f"🏆 Perfect score! You found the optimal path!")
                        print(f"\nPress Enter for the next challenge...")
                        input()
                        break
                    else:
                        print(f"🔄 Try again! You ended at {result.final_position}")
                        if not result.is_valid:
                            print(f"💡 Tip: {result.error_message}")
                
                except KeyboardInterrupt:
                    print(f"\n👋 Thanks for playing!")
                    return
            
            challenge_num += 1
    
    def benchmark_mode(self):
        """Run benchmark tests on multiple challenges."""
        print(f"\n🏃‍♂️ BENCHMARK MODE")
        print(f"{'='*60}")
        
        total_challenges = 5
        results = []
        
        for i in range(total_challenges):
            challenge = self.validator.create_navigation_challenge(seed=(i + 1) * 123)
            if challenge is None:
                continue
            
            print(f"\nChallenge {i + 1}: {challenge.target_object_name}")
            print(f"Distance: {challenge.optimal_path_length} steps")
            print(f"Path: {challenge.optimal_path_arrows}")
            
            # Test the optimal path
            result = self.validator.validate_path(
                challenge.start,
                challenge.target,
                challenge.optimal_path_arrows
            )
            
            results.append((challenge, result))
            
            if result.reached_target and result.efficiency == 1.0:
                print(f"✅ Verified: Optimal path works perfectly")
            else:
                print(f"❌ Error: Optimal path validation failed")
        
        print(f"\n📈 BENCHMARK SUMMARY")
        print(f"{'='*40}")
        print(f"Total challenges: {len(results)}")
        print(f"All optimal paths verified: {all(r[1].efficiency == 1.0 for r in results)}")
        avg_length = sum(r[0].optimal_path_length for r in results) / len(results) if results else 0
        print(f"Average optimal path length: {avg_length:.1f} steps")
    
    def demo_mode(self):
        """Run the built-in demo."""
        demo_pathfinding(self.sim)
    
    def run(self, mode: str = "interactive"):
        """Run the navigation demo in the specified mode."""
        if mode == "interactive":
            self.interactive_challenges()
        elif mode == "benchmark":
            self.benchmark_mode()
        elif mode == "demo":
            self.demo_mode()
        else:
            print(f"❌ Unknown mode: {mode}")


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Navigation Challenge Demo")
    parser.add_argument("--seed", type=int, default=42,
                       help="Random seed for apartment generation (default: 42)")
    parser.add_argument("--width", type=int, default=25,
                       help="Apartment width (default: 25)")
    parser.add_argument("--height", type=int, default=20,
                       help="Apartment height (default: 20)")
    parser.add_argument("--rooms", type=int, default=6,
                       help="Maximum number of rooms (default: 6)")
    parser.add_argument("--objects", type=int, default=20,
                       help="Maximum number of objects (default: 20)")
    parser.add_argument("--mode", choices=["interactive", "benchmark", "demo"], 
                       default="interactive",
                       help="Demo mode: interactive (default), benchmark, or demo")
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
    
    print(f"🎮 Navigation Challenge Demo")
    print(f"Mode: {args.mode}")
    print(f"Apartment: {args.width}x{args.height}, {args.rooms} rooms, {args.objects} objects")
    print(f"Seed: {args.seed}")
    
    # Create and run the demo
    demo = NavigationDemo(opts)
    demo.run(args.mode)


if __name__ == "__main__":
    main()