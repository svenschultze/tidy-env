#!/bin/bash
# Activation script for the tidy-env Python environment

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Activate the virtual environment
source "$PROJECT_ROOT/venv/bin/activate"

echo "🐍 Virtual environment activated!"
echo "📦 tidy_env_py package is ready to use"
echo ""
echo "Available simulators:"
echo "  python interactive_sim.py  - Text-based interactive simulator"
echo "  python visual_sim.py       - Colorful visual simulator"
echo ""
echo "To deactivate the environment, run: deactivate"