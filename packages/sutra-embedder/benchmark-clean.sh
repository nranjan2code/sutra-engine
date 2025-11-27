#!/usr/bin/env bash
# Clean Benchmark Output Script
# 
# Filters out ONNX Runtime's verbose BFCArena allocator logs for clean,
# production-ready benchmark output.
#
# Usage:
#   ./benchmark-clean.sh --profile auto --iterations 100
#   ./benchmark-clean.sh --profile desktop --iterations 50
#
# The BFCArena logs are informational only and come from ONNX Runtime's C++ core.
# They show memory allocation patterns but don't affect performance or indicate errors.
#
# See docs/development/LOGGING.md for more details about ONNX Runtime logging.

# Run benchmark and filter out ONNX Runtime internal logs
./target/release/sutra-embedder benchmark "$@" 2>&1 | \
  grep -v "BFCArena" | \
  grep -v "Extended allocation" | \
  grep -v "Total allocated bytes" | \
  grep -v "Allocated memory at" | \
  grep -v "Reserving memory" | \
  grep -v "Extending BFCArena"
