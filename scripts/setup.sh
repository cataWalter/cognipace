#!/bin/bash
# CogniPace Development Setup Script

set -e

echo "🧠 Setting up CogniPace development environment..."

# Check for required tools
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    fi
    echo "✅ $1 found"
}

echo ""
echo "Checking prerequisites..."
check_command rustc
check_command cargo
check_command python3
check_command npm

# Install Rust dependencies
echo ""
echo "📦 Installing Rust dependencies..."
cargo build --release

# Setup Python environment
echo ""
echo "🐍 Setting up Python environment..."
cd python/training
pip install -r requirements.txt
cd ../..

# Setup dashboard frontend
echo ""
echo "🎨 Setting up dashboard frontend..."
cd crates/cognipace-dashboard
npm install
cd ../..

# Create config directory
echo ""
echo "📁 Creating configuration directory..."
mkdir -p ~/.config/cognipace
cp configs/default.yaml ~/.config/cognipace/config.yaml

echo ""
echo "✅ CogniPace setup complete!"
echo ""
echo "Next steps:"
echo "  1. Run 'cargo run --bin cognipace-cli' to start the CLI"
echo "  2. Run 'cargo run --bin cognipace-dashboard' to start the dashboard"
echo "  3. Train ML models: cd python/training && python train.py --synthetic"
