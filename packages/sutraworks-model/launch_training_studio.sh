#!/bin/bash
# SutraWorks Training Studio Launcher

echo "🚀 Starting SutraWorks Training Studio..."
echo "======================================"
echo ""
echo "A user-friendly GUI for training AI models without requiring ML expertise."
echo ""
echo "Features:"
echo "• 🎯 Model Templates - Choose from pre-configured templates for common use cases"
echo "• 📁 Drag & Drop Data Loading - Simply drop your training data files"
echo "• ⚙️ Visual Configuration - Configure training parameters with sliders and dropdowns"
echo "• 📊 Real-time Progress - Monitor training with live metrics and progress bars"
echo "• 🚀 One-click Training - Start training with a single click"
echo "• 📦 Model Export - Export trained models in multiple formats"
echo ""
echo "System Requirements:"
echo "• macOS (optimized for Apple Silicon)"
echo "• 8GB+ RAM (16GB recommended for larger models)"
echo "• Rust 1.70+ (automatically managed)"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the sutraworks-model root directory"
    exit 1
fi

# Build the training application if needed
echo "🔨 Building Training Studio..."
cargo build --bin sutra-train --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🎨 Launching Training Studio..."
    cargo run --bin sutra-train --release
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi