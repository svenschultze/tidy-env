from pathfinding_validator import ApartmentSimEnv

env = ApartmentSimEnv()
env.reset(seed=123)

print("ASCII Layout:")
print(env.ascii())
print()
challenge = env.get_challenge()
print("Challenge:")
print(f"  Description: {challenge.description}")
print(f"  Start: {challenge.start}")
print(f"  Target: {challenge.target} ({challenge.target_object_name})")
print(f"  Optimal path (arrows): {challenge.optimal_path_arrows}")

# Try the optimal path
result = env.reward(challenge.optimal_path_arrows)
print("\nReward for optimal path:")
print(result)

# Try a suboptimal path (add some wasteful moves)
if len(challenge.optimal_path_arrows) > 0:
    suboptimal = challenge.optimal_path_arrows + "→→←←"
    result2 = env.reward(suboptimal)
    print("\nReward for suboptimal path:")
    print(result2)

# Try an invalid path (walk into a wall)
invalid = "↑↑↑↑↑↑↑↑↑↑"
result3 = env.reward(invalid)
print("\nReward for invalid path:")
print(result3)
