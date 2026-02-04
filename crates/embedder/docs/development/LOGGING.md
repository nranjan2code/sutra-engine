# Logging Configuration & Troubleshooting

## Overview
Sutra Embedder uses multiple logging systems:
- **Rust `tracing`**: Application-level logs (can be controlled)
- **ONNX Runtime C++**: Internal logs from ONNX Runtime library (limited control)

## ONNX Runtime Verbose Logs

### What You're Seeing
When running benchmarks or embeddings, you may see verbose logs like:
```
INFO Creating BFCArena for Cpu with following configs...
INFO Extending BFCArena for Cpu. bin_num:0...
INFO Extended allocation by 1048576 bytes...
INFO Reserving memory in BFCArena for Cpu size: 589824
```

### Why This Happens
These logs come from **ONNX Runtime's internal BFCArena (Best-Fit with Coalescing) memory allocator** at the C++ level. They bypass Rust's logging system and are printed directly to stdout/stderr.

### Why We Can't Suppress Them Completely
- ONNX Runtime is a compiled C++ library
- BFCArena logs are hardcoded in the C++ implementation
- Environment variables like `ORT_LOGGING_LEVEL` only control some logs, not allocator-level ones
- We've already set the log level to FATAL (maximum suppression)

### Are These Logs Harmful?
**No.** These are purely informational logs showing memory allocation patterns. They don't indicate errors or performance issues. You can safely ignore them.

## Solutions

### Option 1: Use the Clean Script (Recommended)
We provide a shell script that filters out ONNX Runtime allocator logs:

```bash
./benchmark-clean.sh --profile auto --iterations 100
```

This gives you clean, production-ready output without the verbose logs.

### Option 2: Manual Filtering
Filter logs yourself using grep:

```bash
./target/release/sutra-embedder benchmark --profile auto --iterations 100 2>&1 | \
  grep -v "BFCArena" | \
  grep -v "Extended allocation" | \
  grep -v "Total allocated bytes" | \
  grep -v "Allocated memory at" | \
  grep -v "Reserving memory" | \
  grep -v "Extending BFCArena"
```

### Option 3: Redirect to /dev/null
If you only care about the benchmark results (not progress):

```bash
./target/release/sutra-embedder benchmark --profile auto --iterations 100 2>/dev/null
```

⚠️ **Warning**: This will also suppress legitimate error messages.

### Option 4: Accept the Logs
For development and debugging, the allocator logs can be useful to understand memory usage patterns. They show:
- When new memory arenas are created
- How much memory is being allocated
- Memory reservation patterns

## Controlling Application Logs

### Set Log Level via Environment
Control Sutra Embedder's own logs (not ONNX Runtime):

```bash
# Show only warnings and errors
RUST_LOG=warn ./target/release/sutra-embedder benchmark --profile auto --iterations 100

# Show info-level logs
RUST_LOG=info ./target/release/sutra-embedder benchmark --profile auto --iterations 100

# Debug mode (very verbose)
RUST_LOG=debug ./target/release/sutra-embedder benchmark --profile auto --iterations 100
```

### Default Behavior
By default, Sutra Embedder is configured to:
- Show WARN level and above for all crates
- Show INFO level for sutra_embedder crate
- Suppress DEBUG and TRACE logs

## FAQ

**Q: Can ONNX Runtime be recompiled without these logs?**
A: Yes, but it would require maintaining a custom ONNX Runtime build, which is not practical for distribution.

**Q: Do these logs impact performance?**
A: Minimally. The logs are I/O operations that happen during model initialization (once per session), not during inference.

**Q: Will future versions suppress these?**
A: We're tracking ONNX Runtime updates. If the upstream library adds better log control, we'll adopt it immediately.

**Q: Can I contribute a fix?**
A: Absolutely! If you find a way to suppress BFCArena logs via ONNX Runtime configuration, please open a PR.

## Technical Details

### Log Levels
ONNX Runtime supports these log levels:
- `0` = VERBOSE
- `1` = INFO (BFCArena logs are here)
- `2` = WARNING
- `3` = ERROR  
- `4` = FATAL

We set `ORT_LOGGING_LEVEL=4` (FATAL only), but BFCArena logs bypass this setting.

### Environment Variables Tried
We've attempted these configurations:
- `ORT_LOGGING_LEVEL=4`
- `ORT_LOG_SEVERITY_LEVEL=4`
- `TF_CPP_MIN_LOG_LEVEL=3`
- Session builder `with_log_level(LogLevel::Fatal)`

None completely suppress the allocator logs.

## Summary
**For Clean Output**: Use `./benchmark-clean.sh`  
**For Development**: Accept the logs - they're informational  
**For Production**: Pipe through grep filters or redirect stderr

The logs don't indicate a problem with Sutra Embedder - they're a normal part of ONNX Runtime's operation.
