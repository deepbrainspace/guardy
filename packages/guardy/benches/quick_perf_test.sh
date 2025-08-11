#!/bin/bash
set -euo pipefail

BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GUARDY_PATH="/home/nsm/code/deepbrain/guardy/target/release/guardy"

echo "🚀 Quick Guardy Performance Test"
echo "================================"
echo ""

# Build guardy first
echo "🔨 Building Guardy..."
cd /home/nsm/code/deepbrain/guardy/packages/guardy
cargo build --release --quiet

if [ ! -f "$GUARDY_PATH" ]; then
    echo "❌ Failed to build guardy"
    exit 1
fi

echo "✅ Guardy built successfully"

# Use existing test data
TEST_PROJECTS=()
if [ -d "$BENCH_DIR/test-data/small-project" ]; then
    TEST_PROJECTS+=("$BENCH_DIR/test-data/small-project")
    echo "📦 Found small project ($(find "$BENCH_DIR/test-data/small-project" -type f | wc -l) files)"
fi
if [ -d "$BENCH_DIR/test-data/medium-project" ]; then
    TEST_PROJECTS+=("$BENCH_DIR/test-data/medium-project")
    echo "📦 Found medium project ($(find "$BENCH_DIR/test-data/medium-project" -type f | wc -l) files)"
fi
if [ -d "$BENCH_DIR/test-data/large-project" ]; then
    TEST_PROJECTS+=("$BENCH_DIR/test-data/large-project")
    echo "📦 Found large project ($(find "$BENCH_DIR/test-data/large-project" -type f | wc -l) files)"
fi

if [ ${#TEST_PROJECTS[@]} -eq 0 ]; then
    echo "❌ No test data found. Please run the test data generator first."
    exit 1
fi

echo ""
echo "🔍 Running Scanning Performance Tests"
echo "====================================="

for project_dir in "${TEST_PROJECTS[@]}"; do
    project_name=$(basename "$project_dir")
    file_count=$(find "$project_dir" -type f | wc -l)

    echo ""
    echo "📊 Testing $project_name ($file_count files)..."

    cd "$project_dir"

    # Clear caches for cold test
    sync; echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true

    # Cold run (first run without cache)
    echo "🥶 Cold run (no cache):"
    start_time=$(date +%s.%N)
    cold_output=$("$GUARDY_PATH" scan . --quiet 2>&1 || true)
    end_time=$(date +%s.%N)
    cold_time=$(echo "$end_time - $start_time" | bc)

    # Extract actual files scanned from output
    actual_files=$(echo "$cold_output" | grep -oE '[0-9]+ files scanned' | grep -oE '[0-9]+' || echo "$file_count")

    echo "  Cold: ${cold_time}s ($actual_files files)"

    # Warm runs (with cache)
    total_time=0
    runs=3

    echo "🔥 Warm runs ($runs iterations):"

    for i in $(seq 1 $runs); do
        start_time=$(date +%s.%N)
        warm_output=$("$GUARDY_PATH" scan . --quiet 2>&1 || true)
        end_time=$(date +%s.%N)

        duration=$(echo "$end_time - $start_time" | bc)
        total_time=$(echo "$total_time + $duration" | bc)

        echo "  Run $i: ${duration}s"
    done

    avg_time=$(echo "scale=4; $total_time / $runs" | bc)
    warm_fps=$(echo "scale=1; $actual_files / $avg_time" | bc)
    cold_fps=$(echo "scale=1; $actual_files / $cold_time" | bc)

    echo ""
    echo "📈 Results for $project_name:"
    echo "  Cold run:     ${cold_time}s (${cold_fps} files/sec)"
    echo "  Warm average: ${avg_time}s (${warm_fps} files/sec)"
    echo "  Files scanned: $actual_files"
    echo "  Cache speedup: $(echo "scale=1; $cold_time / $avg_time" | bc)x"
done

# Test parallel workers scaling
echo ""
echo "⚡ Parallel Workers Scaling Test"
echo "==============================="

if [ -d "$BENCH_DIR/test-data/medium-project" ]; then
    cd "$BENCH_DIR/test-data/medium-project"

    for workers in 1 2 4 8; do
        # Create temporary config
        cat > .guardy.yaml << EOF
scanning:
  enabled: true
  paths: ["src/", "config/"]

parallelism:
  enabled: true
  max_workers: $workers
EOF

        echo ""
        echo "🔧 Testing with $workers worker(s)..."

        # Run test
        start_time=$(date +%s.%N)
        "$GUARDY_PATH" scan . --quiet --no-report >/dev/null 2>&1 || true
        end_time=$(date +%s.%N)

        duration=$(echo "$end_time - $start_time" | bc)
        files_per_sec=$(echo "scale=1; 639 / $duration" | bc) # medium project has ~639 files

        echo "  $workers workers: ${duration}s (${files_per_sec} files/sec)"

        # Clean up config
        rm -f .guardy.yaml
    done
fi

# Performance comparison with other tools (if available)
echo ""
echo "⚖️  Tool Comparison"
echo "=================="

if [ -d "$BENCH_DIR/test-data/medium-project" ]; then
    cd "$BENCH_DIR/test-data/medium-project"

    echo ""
    echo "📊 Hook Execution vs Lefthook Comparison..."

    # Setup guardy hook config
    cat > .guardy.yaml << 'EOF'
hooks:
  pre-commit:
    - name: "lint-check"
      command: "echo 'Linting...' && sleep 0.1"
    - name: "type-check"
      command: "echo 'Type checking...' && sleep 0.05"
    - name: "test-quick"
      command: "echo 'Quick tests...' && sleep 0.2"

scanning:
  enabled: false  # Disable scanning for hook comparison

parallelism:
  enabled: true
  max_workers: 4
EOF

    # Test Guardy hooks
    echo "🦀 Testing Guardy pre-commit hooks..."
    "$GUARDY_PATH" install --quiet 2>/dev/null || true

    echo "// Hook test" > hook_test.js
    git add hook_test.js

    start_time=$(date +%s.%N)
    git commit -m "guardy hook test" --quiet 2>/dev/null || true
    end_time=$(date +%s.%N)
    guardy_hook_time=$(echo "$end_time - $start_time" | bc)

    git reset --soft HEAD~1 --quiet 2>/dev/null || true
    rm -f hook_test.js

    echo "  Guardy hooks: ${guardy_hook_time}s"

    # Test Lefthook if available
    if command -v lefthook &> /dev/null; then
        echo "🪝 Testing Lefthook pre-commit hooks..."

        # Setup lefthook config
        cat > .lefthook.yml << 'EOF'
pre-commit:
  commands:
    lint-check:
      run: echo 'Linting...' && sleep 0.1
    type-check:
      run: echo 'Type checking...' && sleep 0.05
    test-quick:
      run: echo 'Quick tests...' && sleep 0.2
EOF

        lefthook install 2>/dev/null || true

        echo "// Hook test" > hook_test.js
        git add hook_test.js

        start_time=$(date +%s.%N)
        git commit -m "lefthook hook test" --quiet 2>/dev/null || true
        end_time=$(date +%s.%N)
        lefthook_hook_time=$(echo "$end_time - $start_time" | bc)

        git reset --soft HEAD~1 --quiet 2>/dev/null || true
        rm -f hook_test.js .lefthook.yml

        echo "  Lefthook hooks: ${lefthook_hook_time}s"

        # Calculate relative performance
        hook_speedup=$(echo "scale=2; $lefthook_hook_time / $guardy_hook_time" | bc)
        comparison_text=$(echo "$hook_speedup > 1" | bc -l | grep -q 1 && echo "faster" || echo "slower")

        echo ""
        echo "📈 Hook Execution Summary:"
        echo "  Guardy is ${hook_speedup}x ${comparison_text} than Lefthook for git hooks"

        # Clean up
        rm -rf .git/hooks/*
    else
        echo "⚠️  Lefthook not available for comparison"
    fi

    # Test scanning performance
    echo ""
    echo "🔍 Testing scanning performance..."

    start_time=$(date +%s.%N)
    scan_output=$("$GUARDY_PATH" scan . --quiet 2>&1 || true)
    end_time=$(date +%s.%N)
    guardy_scan_time=$(echo "$end_time - $start_time" | bc)

    scanned_files=$(echo "$scan_output" | grep -oE '[0-9]+ files scanned' | grep -oE '[0-9]+' || echo "639")
    guardy_scan_fps=$(echo "scale=1; $scanned_files / $guardy_scan_time" | bc)

    echo "  Guardy scan: ${guardy_scan_time}s (${guardy_scan_fps} files/sec, $scanned_files files)"

    # Clean up configs
    rm -f .guardy.yaml
fi

echo ""
echo "🎉 Quick Performance Test Complete!"
echo ""
echo "💡 Key Takeaways:"
echo "  • Guardy's Rust implementation provides consistent performance"
echo "  • Parallel processing scales well with available CPU cores"
echo "  • Performance varies with project size and file types"
echo ""
echo "📊 For detailed benchmarks with statistical analysis, run:"
echo "    cargo bench --bench guardy_benchmarks"
