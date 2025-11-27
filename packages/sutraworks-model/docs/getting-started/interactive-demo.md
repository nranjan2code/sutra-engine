# Quick Start: Interactive Demo

## Experience SutraWorks in Action

The fastest way to understand SutraWorks is to see it working. Our interactive demo showcases all key features in a real-time GUI application.

### 🚀 Launch the Demo (30 seconds)

```bash
# Clone and launch
git clone https://github.com/nranjan2code/sutraworks-model
cd sutraworks-model
./launch_demo.sh
```

### 🎮 What You'll Experience

#### 💬 **AI Chat Interface**
- Chat directly with RWKV and Mamba models
- See real inference times (1-3ms on MacBook Air)
- Switch between architectures instantly
- Watch linear complexity in action

#### 🏎️ **Performance Race**
- Compare RWKV vs Mamba head-to-head
- Real-time benchmarking with different inputs
- See O(n) complexity advantages
- Measure actual tokens per second

#### ⚡ **Live Quantization**
- Compress models in real-time
- Watch 7.42x compression happen
- Adjust quantization parameters live
- See memory savings (86.5% reduction)

#### 🧠 **Neuro-Symbolic Preview**
- Neural + symbolic reasoning
- Tool integration demonstration
- Logic verification pipeline

### 🎯 Key Takeaways

After 5 minutes with the demo, you'll understand:

- **Pure Rust Benefits**: Single binary, memory safe, fast startup
- **Linear Complexity**: RWKV/Mamba vs quadratic transformers
- **Real Compression**: Measured 7.42x ratios, not theoretical
- **Production Ready**: Working code, not research prototypes

### 🚀 Next Steps

1. **Explore the GUI**: Try all four demo tabs
2. **Read the docs**: [Interactive Demo Guide](../interactive-demo/README.md)
3. **Run examples**: `cargo run --example end_to_end --release`
4. **Build your own**: Follow the [API tutorials](../api/core.md)

---

**Ready to dive deeper?** → [Complete Demo Documentation](../interactive-demo/README.md)