#!/usr/bin/env python3
"""
Pathfinding Validator

A comprehensive Python-only pathfinding and navigation validator for the tidy-env apartment simulator.
This module provides functionality to validate movement paths, test pathfinding algorithms,
and create navigation challenges without modifying the core Rust package.

Features:
- Path validation with detailed feedback
- A* pathfinding implementation
- Navigation challenge generation
- Path efficiency analysis
- Interactive demos

Usage:
    from pathfinding_validator import PathfindingValidator, demo_pathfinding
    
    # Create validator
    validator = PathfindingValidator(simulator)
    
    # Validate a path
    result = validator.validate_path(start, target, "↑→→↓")
    
    # Run demo
    demo_pathfinding(simulator)
"""

import sys
import os
import heapq
import random
from typing import List, Tuple, Optional, NamedTuple, Set
from dataclasses import dataclass
from enum import Enum

# Try to import tidy_env_py
import tidy_env_py
from tidy_env_py import constants

class Direction(Enum):
    """Cardinal directions for movement."""
    UP = (0, -1, "↑")
    DOWN = (0, 1, "↓") 
    LEFT = (-1, 0, "←")
    RIGHT = (1, 0, "→")
    
    def __init__(self, dx: int, dy: int, symbol: str):
        self.dx = dx
        self.dy = dy
        self.symbol = symbol
    
    @classmethod
    def from_symbol(cls, symbol: str) -> Optional['Direction']:
        """Get direction from arrow symbol."""
        for direction in cls:
            if direction.symbol == symbol:
                return direction
        return None
    
    @classmethod
    def all_symbols(cls) -> str:
        """Get all direction symbols as a string."""
        return "".join(d.symbol for d in cls)


@dataclass
class PathResult:
    """Result of path validation."""
    start_position: Tuple[int, int]
    target_position: Tuple[int, int]
    final_position: Tuple[int, int]
    path_taken: List[Tuple[int, int]]
    is_valid: bool
    reached_target: bool
    steps_taken: int
    error_message: Optional[str] = None
    efficiency: Optional[float] = None
    optimal_length: Optional[int] = None


@dataclass
class NavigationChallenge:
    """A navigation challenge with start, target, and optimal solution."""
    start: Tuple[int, int]
    target: Tuple[int, int]
    target_object_name: str
    optimal_path: List[Tuple[int, int]]
    optimal_path_length: int
    optimal_path_arrows: str
    description: str


class PathfindingValidator:
    """Comprehensive pathfinding validator for apartment navigation."""
    
    def __init__(self, simulator: tidy_env_py.PySimulator):
        """Initialize with a simulator instance."""
        self.sim = simulator
        self.layout = simulator.get_layout()
        
    def is_walkable(self, x: int, y: int, allow_closed_doors: bool = False) -> bool:
        """Check if a position is walkable (not wall or outside). Optionally allow closed doors."""
        if x < 0 or x >= self.layout.width or y < 0 or y >= self.layout.height:
            return False
        
        cell_value = self.layout.get_cell(x, y)
        
        if allow_closed_doors:
            # Treat closed doors as walkable for pathfinding
            return cell_value >= 0 or cell_value == constants.OPEN_DOOR or cell_value == constants.CLOSED_DOOR
        else:
            return cell_value >= 0 or cell_value == constants.OPEN_DOOR

    def get_neighbors(self, pos: Tuple[int, int], allow_closed_doors: bool = False) -> List[Tuple[int, int]]:
        """Get valid neighboring positions."""
        x, y = pos
        neighbors = []
        
        for direction in Direction:
            new_x = x + direction.dx
            new_y = y + direction.dy
            
            if self.is_walkable(new_x, new_y, allow_closed_doors=allow_closed_doors):
                neighbors.append((new_x, new_y))
        
        return neighbors

    def manhattan_distance(self, pos1: Tuple[int, int], pos2: Tuple[int, int]) -> int:
        """Calculate Manhattan distance between two positions."""
        return abs(pos1[0] - pos2[0]) + abs(pos1[1] - pos2[1])
    
    def find_path_astar(self, start: Tuple[int, int], goal: Tuple[int, int]) -> Optional[List[Tuple[int, int]]]:
        """Find optimal path using A* algorithm, treating closed doors as walkable."""
        if start == goal:
            return [start]
        
        if not self.is_walkable(goal[0], goal[1], allow_closed_doors=True):
            return None
        
        # Priority queue: (f_score, g_score, position, path)
        open_set = [(0, 0, start, [start])]
        visited: Set[Tuple[int, int]] = set()
        
        while open_set:
            f_score, g_score, current, path = heapq.heappop(open_set)
            
            if current in visited:
                continue
            
            visited.add(current)
            
            if current == goal:
                return path
            
            for neighbor in self.get_neighbors(current, allow_closed_doors=True):
                if neighbor in visited:
                    continue
                
                new_g_score = g_score + 1
                new_path = path + [neighbor]
                h_score = self.manhattan_distance(neighbor, goal)
                new_f_score = new_g_score + h_score
                
                heapq.heappush(open_set, (new_f_score, new_g_score, neighbor, new_path))
        
        return None
    
    def path_to_arrows(self, path: List[Tuple[int, int]]) -> str:
        """Convert a path to arrow notation."""
        if len(path) < 2:
            return ""
        
        arrows = []
        for i in range(1, len(path)):
            prev_x, prev_y = path[i-1]
            curr_x, curr_y = path[i]
            
            dx = curr_x - prev_x
            dy = curr_y - prev_y
            
            for direction in Direction:
                if direction.dx == dx and direction.dy == dy:
                    arrows.append(direction.symbol)
                    break
        
        return "".join(arrows)
    
    def validate_path(self, start: Tuple[int, int], target: Tuple[int, int], 
                     arrow_path: str) -> PathResult:
        """Validate a path given as arrow notation."""
        
        # Initialize result
        result = PathResult(
            start_position=start,
            target_position=target,
            final_position=start,
            path_taken=[start],
            is_valid=True,
            reached_target=False,
            steps_taken=0
        )
        
        # Check if start position is valid
        if not self.is_walkable(start[0], start[1]):
            result.is_valid = False
            result.error_message = f"Start position {start} is not walkable"
            return result
        
        # Check if target position is valid
        if not self.is_walkable(target[0], target[1]):
            result.is_valid = False
            result.error_message = f"Target position {target} is not walkable"
            return result
        
        # Execute the path step by step
        current_x, current_y = start
        
        for i, symbol in enumerate(arrow_path):
            direction = Direction.from_symbol(symbol)
            if direction is None:
                result.is_valid = False
                result.error_message = f"Invalid direction symbol '{symbol}' at position {i}"
                break
            
            new_x = current_x + direction.dx
            new_y = current_y + direction.dy
            
            cell_value = self.layout.get_cell(new_x, new_y)
            # If it's a closed door, simulate opening it before moving
            if cell_value == constants.CLOSED_DOOR:
                # Simulate opening the door (in real sim, would require an action)
                # For validation, just allow the move
                pass  # Door is now open for this step
            elif not self.is_walkable(new_x, new_y):
                result.is_valid = False
                result.error_message = f"Cannot move {direction.symbol} from ({current_x}, {current_y}) to ({new_x}, {new_y}) - blocked"
                break
            
            # Execute the move
            current_x, current_y = new_x, new_y
            result.path_taken.append((current_x, current_y))
            result.steps_taken += 1
        
        result.final_position = (current_x, current_y)
        result.reached_target = (result.final_position == target)
        
        # Calculate efficiency if we have a valid result
        if result.is_valid:
            optimal_path = self.find_path_astar(start, target)
            if optimal_path:
                result.optimal_length = len(optimal_path) - 1  # Number of steps
                if result.steps_taken > 0:
                    result.efficiency = result.optimal_length / result.steps_taken
                else:
                    result.efficiency = 1.0 if result.reached_target else 0.0
        
        return result
    
    def create_navigation_challenge(self, seed: Optional[int] = None) -> Optional[NavigationChallenge]:
        """Create a navigation challenge with a random object as target."""
        if seed is not None:
            random.seed(seed)
        
        # Get all objects in the world
        objects = self.sim.get_objects()
        if not objects:
            return None
        
        # Filter out contained objects (only use objects that appear in "Objects in apartment" list)
        excluded_ids = set()
        for obj in objects:
            if hasattr(obj, 'contents'):
                excluded_ids.update(obj.contents)
        
        # Only consider objects that are not contained within other objects
        available_objects = [obj for obj in objects if obj.id not in excluded_ids]
        
        if not available_objects:
            return None
        
        # Use current agent position as start
        start_pos = (self.sim.agent_x, self.sim.agent_y)
        
        # Test pathfinding to each available object before choosing one
        valid_targets = []
        for obj in available_objects:
            target_pos = (obj.x, obj.y)
            optimal_path = self.find_path_astar(start_pos, target_pos)
            if optimal_path and len(optimal_path) >= 2:
                valid_targets.append(obj)
        
        if not valid_targets:
            return None
        
        # Choose a random object from the valid targets
        target_obj = random.choice(valid_targets)
        target_pos = (target_obj.x, target_obj.y)
        
        # Find optimal path
        optimal_path = self.find_path_astar(start_pos, target_pos)
        if not optimal_path or len(optimal_path) < 2:
            return None
        
        optimal_arrows = self.path_to_arrows(optimal_path)
        
        # Create challenge description
        distance = len(optimal_path) - 1
        description = f"Navigate to the {target_obj.name}"
        
        return NavigationChallenge(
            start=start_pos,
            target=target_pos,
            target_object_name=target_obj.name,
            optimal_path=optimal_path,
            optimal_path_length=distance,
            optimal_path_arrows=optimal_arrows,
            description=description
        )
    
    def analyze_path_efficiency(self, result: PathResult) -> str:
        """Provide human-readable analysis of path efficiency."""
        if not result.is_valid:
            return f"❌ Invalid path: {result.error_message}"
        
        if not result.reached_target:
            return f"🎯 Did not reach target. Ended at {result.final_position}"
        
        if result.efficiency is None:
            return "✅ Reached target (efficiency unknown)"
        
        if result.efficiency >= 1.0:
            return f"🏆 Perfect! Optimal path in {result.steps_taken} steps"
        elif result.efficiency >= 0.8:
            return f"⭐ Excellent! {result.efficiency:.1%} efficiency ({result.steps_taken} steps, optimal: {result.optimal_length})"
        elif result.efficiency >= 0.6:
            return f"👍 Good path. {result.efficiency:.1%} efficiency ({result.steps_taken} steps, optimal: {result.optimal_length})"
        elif result.efficiency >= 0.4:
            return f"📈 Could be better. {result.efficiency:.1%} efficiency ({result.steps_taken} steps, optimal: {result.optimal_length})"
        else:
            return f"🔄 Try a shorter route. {result.efficiency:.1%} efficiency ({result.steps_taken} steps, optimal: {result.optimal_length})"

    def print_apartment_layout(self) -> None:
        """Print the full apartment ASCII layout with agent and objects."""
        print("🏠 Apartment Layout:")
        print("=" * 50)
        
        width = self.layout.width
        height = self.layout.height
        
        # Create the grid
        grid = []
        for y in range(height):
            row = []
            for x in range(width):
                cell_value = self.layout.get_cell(x, y)
                # Map cell value to character
                if cell_value < 0:
                    if cell_value == constants.WALL:
                        char = '■'  # wall
                    elif cell_value == constants.OUTSIDE:
                        char = ' '  # outside
                    elif cell_value == constants.CLOSED_DOOR:
                        char = 'D'  # closed door
                    elif cell_value == constants.OPEN_DOOR:
                        char = 'd'  # open door
                    else:
                        char = '?'  # unknown
                else:
                    # room cell: empty square symbol
                    char = '□'
                row.append(char)
            grid.append(row)
        
        # Place objects with their IDs (avoiding contained objects)
        objects = self.sim.get_objects()
        excluded_ids = set()
        for obj in objects:
            if hasattr(obj, 'contents'):
                excluded_ids.update(obj.contents)
        
        for obj in objects:
            if obj.id not in excluded_ids and 0 <= obj.x < width and 0 <= obj.y < height:
                grid[obj.y][obj.x] = str(obj.id)
        
        # Place agent as '@'
        agent_x = self.sim.agent_x
        agent_y = self.sim.agent_y
        if 0 <= agent_x < width and 0 <= agent_y < height:
            grid[agent_y][agent_x] = '@'
        
        # Print the grid
        for row in grid:
            print(''.join(row))
        
        print()
        print("Legend:")
        print("  @ = Agent")
        print("  ■ = Wall")
        print("  □ = Room (walkable)")
        print("  D = Closed door")
        print("  d = Open door")
        print("  0-9 = Objects (by ID)")
        print("    = Outside")
        print()
        print(f"Agent position: ({agent_x}, {agent_y})")
        
        # Print object details
        if objects:
            print("\nObjects in apartment:")
            for obj in objects:
                if obj.id not in excluded_ids:
                    print(f"  {obj.id}: {obj.name} at ({obj.x}, {obj.y})")
        print()


def demo_pathfinding(simulator: tidy_env_py.PySimulator) -> None:
    """Run a demonstration of the pathfinding validator."""
    print("🗺️ Pathfinding Demo")
    print("=" * 50)
    
    validator = PathfindingValidator(simulator)
    
    # Print the apartment layout
    validator.print_apartment_layout()
    
    # Try to create a navigation challenge
    challenge = validator.create_navigation_challenge(seed=42)
    
    if challenge is None:
        print("❌ Could not create navigation challenge (no objects or no path).")
        return
    
    print(f"📍 Challenge: {challenge.description}")
    print(f"🎯 Target: {challenge.target_object_name} at {challenge.target}")
    print(f"🏁 Optimal path: {challenge.optimal_path_arrows} ({challenge.optimal_path_length} steps)")
    
    # Test the optimal path
    result = validator.validate_path(
        challenge.start, 
        challenge.target, 
        challenge.optimal_path_arrows
    )
    
    analysis = validator.analyze_path_efficiency(result)
    print(f"✅ Validation: {analysis}")
    
    # Test a suboptimal path by adding extra moves
    if len(challenge.optimal_path_arrows) > 0:
        suboptimal_path = challenge.optimal_path_arrows + "→←"  # Add wasteful moves
        print(f"\n🔄 Testing suboptimal path: {suboptimal_path}")
        
        result2 = validator.validate_path(
            challenge.start,
            challenge.target,
            suboptimal_path
        )
        
        analysis2 = validator.analyze_path_efficiency(result2)
        print(f"📊 Result: {analysis2}")
    
    # Test an invalid path
    print(f"\n❌ Testing invalid path: ↑↑↑↑↑↑↑↑↑↑")
    result3 = validator.validate_path(
        challenge.start,
        challenge.target,
        "↑↑↑↑↑↑↑↑↑↑"
    )
    
    analysis3 = validator.analyze_path_efficiency(result3)
    print(f"📊 Result: {analysis3}")


class ApartmentSimEnv:
    """
    High-level environment wrapper for apartment navigation tasks.
    Usage:
        env = ApartmentSimEnv()
        env.reset(seed=123)
        print(env.ascii())
        challenge = env.get_challenge()
        score = env.reward('↓↓→→→')
    """
    def __init__(self, width=20, height=15, max_rooms=5, max_objects=10):
        self.width = width
        self.height = height
        self.max_rooms = max_rooms
        self.max_objects = max_objects
        self.sim = None
        self.validator = None
        self.challenge = None
        self._ascii = None

    def reset(self, seed=None):
        """Generate a new random layout and challenge."""
        opts = tidy_env_py.PyGenOpts(
            seed=seed if seed is not None else random.randint(0, 1 << 30),
            max_rooms=self.max_rooms,
            width=self.width,
            height=self.height,
            max_objects=self.max_objects
        )
        self.sim = tidy_env_py.PySimulator(opts)
        self.validator = PathfindingValidator(self.sim)
        self.challenge = self.validator.create_navigation_challenge(seed=seed)
        # Generate ASCII layout
        ascii_lines = []
        width = self.validator.layout.width
        height = self.validator.layout.height
        grid = []
        for y in range(height):
            row = []
            for x in range(width):
                cell_value = self.validator.layout.get_cell(x, y)
                if cell_value < 0:
                    if cell_value == constants.WALL:
                        char = '■'
                    elif cell_value == constants.OUTSIDE:
                        char = ' '
                    elif cell_value == constants.CLOSED_DOOR:
                        char = 'D'
                    elif cell_value == constants.OPEN_DOOR:
                        char = 'd'
                    else:
                        char = '?'
                else:
                    char = '□'
                row.append(char)
            grid.append(row)
        # Place objects
        objects = self.sim.get_objects()
        excluded_ids = set()
        for obj in objects:
            if hasattr(obj, 'contents'):
                excluded_ids.update(obj.contents)
        for obj in objects:
            if obj.id not in excluded_ids and 0 <= obj.x < width and 0 <= obj.y < height:
                grid[obj.y][obj.x] = str(obj.id)
        # Place agent
        agent_x = self.sim.agent_x
        agent_y = self.sim.agent_y
        if 0 <= agent_x < width and 0 <= agent_y < height:
            grid[agent_y][agent_x] = '@'
        for row in grid:
            ascii_lines.append(''.join(row))
        self._ascii = '\n'.join(ascii_lines)
        return self

    def ascii(self):
        """Return the ASCII layout string."""
        return self._ascii

    def get_challenge(self):
        """Return the current challenge object."""
        return self.challenge

    def reward(self, path_arrows: str):
        """Evaluate a path for the current challenge. Returns a dict with is_valid, efficiency, cross_score."""
        if self.sim is None or self.challenge is None:
            raise RuntimeError("Call reset() before reward().")
        result = self.validator.validate_path(self.challenge.start, self.challenge.target, path_arrows)
        is_valid = 1 if result.is_valid else 0
        efficiency = result.efficiency if result.is_valid else 0
        cross_score = None
        if not result.is_valid:
            optimal_path = self.validator.find_path_astar(self.challenge.start, self.challenge.target)
            if optimal_path and len(optimal_path) > 1:
                optimal_set = set(optimal_path)
                last_cross = -1
                for idx, pos in enumerate(result.path_taken):
                    if pos in optimal_set:
                        last_cross = idx
                if last_cross > 0:
                    last_pos = result.path_taken[last_cross]
                    try:
                        opt_idx = optimal_path.index(last_pos)
                        cross_score = opt_idx / (len(optimal_path) - 1)
                    except Exception:
                        cross_score = 0.0
                else:
                    cross_score = 0.0
        else:
            cross_score = 1.0  # If valid, assume full cross score
        return {
            'is_valid': is_valid,
            'efficiency': efficiency,
            'cross_score': cross_score
        }