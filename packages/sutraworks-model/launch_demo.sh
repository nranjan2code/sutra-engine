#!/bin/bash

echo "🚀 Launching SutraWorks AI Interactive Demo..."
echo "═════════════════════════════════════════════"
echo ""
echo "Features:"
echo "• 💬 Real-time chat with RWKV models"
echo "• 🏎️  Architecture performance comparison"
echo "• ⚡ Live quantization demonstration"  
echo "• 🧠 Neuro-symbolic reasoning preview"
echo ""
echo "Pure Rust • Zero Dependencies • Edge Optimized"
echo ""

# Build the demo in release mode for best performance
cargo build --bin sutra-demo --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful! Starting interactive demo..."
    echo ""
    
    # Run the demo
    cargo run --bin sutra-demo --release
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi