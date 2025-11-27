#!/bin/bash
# Comprehensive Benchmark Runner - Quick Examples

set -e

echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║        Sutra Embedder - Comprehensive Benchmark Quick Examples              ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Build first
echo "📦 Building in release mode..."
cargo build --release --quiet
echo "✓ Build complete"
echo ""

# Example 1: Quick test with 3 dimensions
echo "Example 1: Quick benchmark (3 dimensions, 20 iterations)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/sutra-embedder comprehensive-benchmark \
  --dimensions "256,384,768" \
  --iterations 20 \
  --output-dir benchmark_quick
echo ""

# Example 2: Balanced benchmark (5 dimensions)
echo "Example 2: Balanced benchmark (5 dimensions, 50 iterations)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/sutra-embedder comprehensive-benchmark \
  --dimensions "128,256,384,512,768" \
  --iterations 50 \
  --output-dir benchmark_balanced
echo ""

# Example 3: IoT/Edge focus (small dimensions)
echo "Example 3: IoT/Edge benchmark (small dimensions, high iterations)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
./target/release/sutra-embedder comprehensive-benchmark \
  --dimensions "64,128,256" \
  --iterations 100 \
  --output-dir benchmark_iot
echo ""

# Show results
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                            Results Summary                                   ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""

for dir in benchmark_quick benchmark_balanced benchmark_iot; do
  if [ -f "$dir/benchmark_results.csv" ]; then
    echo "📊 $dir results:"
    echo "   JSON:     $dir/benchmark_results.json"
    echo "   CSV:      $dir/benchmark_results.csv"
    echo "   Markdown: $dir/benchmark_report.md"
    echo ""
    
    # Show quick stats from CSV
    echo "   Quick Stats:"
    tail -n +2 "$dir/benchmark_results.csv" | awk -F',' '{
      printf "     %dD: Coherence=%.1f%%, Latency=%.2fms, Throughput=%.0f/s\n", 
             $1, $3*100, $6, $11
    }'
    echo ""
  fi
done

echo "✅ All benchmarks complete!"
echo ""
echo "💡 Tips:"
echo "   • View Markdown reports for detailed analysis"
echo "   • Import CSV into Excel/Google Sheets for charts"
echo "   • Use JSON for programmatic analysis"
echo "   • Run with more iterations (-i 200) for production results"
echo ""
echo "📖 See docs/benchmarks/BENCHMARKS.md for interpretation guide"
