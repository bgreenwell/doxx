# Performance Targets & Optimization

Performance benchmarks, targets, and profiling guidance for doxx.

## Performance Targets

### ✅ Achieved Targets

| Metric | Target | Status | Notes |
|--------|--------|--------|-------|
| **Startup time** | < 100ms | ✅ Achieved | Cold start on typical hardware |
| **Memory usage** | < 50MB | ✅ Achieved | For typical documents (10-50 pages) |
| **Rendering speed** | < 500ms | ✅ Achieved | Complex documents with tables/equations |
| **Binary size** | ~3MB | ✅ Achieved | With LTO optimization |

### Context & Rationale

**Why these targets?**
- **Startup time**: Must feel instant for terminal workflows
- **Memory usage**: Respect system resources, enable multiple instances
- **Rendering speed**: Near-instant display, no perceived lag
- **Binary size**: Quick download, minimal disk footprint

**Comparison:**
- Microsoft Word startup: 8+ seconds
- LibreOffice startup: 5+ seconds
- doxx startup: < 100ms (80x faster)

## Measuring Performance

### Startup Time

```bash
# Using time command
time cargo run --release -- tests/fixtures/minimal.docx --export text > /dev/null

# More accurate with hyperfine
cargo install hyperfine
hyperfine 'cargo run --release -- tests/fixtures/minimal.docx --export text'
```

### Memory Usage

```bash
# Using /usr/bin/time (Linux)
/usr/bin/time -v cargo run --release -- tests/fixtures/business-report.docx

# Using time (macOS)
/usr/bin/time -l cargo run --release -- tests/fixtures/business-report.docx

# With valgrind (Linux)
valgrind --tool=massif cargo run --release -- tests/fixtures/business-report.docx
ms_print massif.out.*
```

### Rendering Performance

```bash
# Profile with built-in benchmarks
cargo bench

# With perf (Linux)
cargo build --release
perf record --call-graph=dwarf ./target/release/doxx tests/fixtures/large.docx
perf report

# With Instruments (macOS)
cargo build --release
instruments -t "Time Profiler" ./target/release/doxx tests/fixtures/large.docx
```

### Binary Size

```bash
# Check size
ls -lh target/release/doxx

# Strip symbols (already done by default)
strip target/release/doxx
ls -lh target/release/doxx

# Analyze with cargo-bloat
cargo install cargo-bloat
cargo bloat --release
```

## Optimization Techniques

### Compilation Optimizations

**Cargo.toml profile settings:**
```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower compile
opt-level = 3           # Maximum optimization
strip = true            # Remove debug symbols
```

### Runtime Optimizations

1. **Lazy loading**: Don't parse entire document upfront
2. **Caching**: Memoize expensive computations (equation rendering)
3. **Async I/O**: Use Tokio for file operations
4. **Zero-copy**: Minimize allocations and copies
5. **String interning**: Reuse common strings (styles, fonts)

### Common Bottlenecks

**Document parsing:**
- XML parsing (docx-rs)
- Equation conversion (OMML to LaTeX)
- Image decoding

**Rendering:**
- Unicode segmentation
- ANSI escape code generation
- Table layout calculations

**Terminal I/O:**
- Terminal querying (size, capabilities)
- Image rendering (Kitty, iTerm2 protocols)

## Profiling Workflow

### 1. Identify Bottleneck
```bash
# Profile with perf
perf record --call-graph=dwarf ./target/release/doxx tests/fixtures/large.docx
perf report

# Look for hot functions (> 5% samples)
```

### 2. Reproduce Minimally
```bash
# Create minimal test case
# Isolate specific operation
```

### 3. Benchmark Current
```bash
# Establish baseline
cargo bench -- specific_bench
```

### 4. Optimize
```bash
# Implement optimization
# Measure again
cargo bench -- specific_bench
```

### 5. Verify
```bash
# Run full test suite
cargo test --all-features

# Check CI/CD still passes
./scripts/check.sh
```

## Performance Testing

### Test Fixtures

**Small**: `tests/fixtures/minimal.docx` (1 page, ~5KB)
**Medium**: `tests/fixtures/business-report.docx` (10 pages, ~50KB)
**Large**: `tests/fixtures/technical-manual.docx` (100+ pages, ~2MB)
**Complex**: `tests/fixtures/equations.docx` (heavy math, tables)

### Benchmark Suite

```bash
cargo bench
```

**Benchmarks:**
- Document parsing (various sizes)
- Equation rendering (OMML to LaTeX)
- Table layout calculations
- ANSI export generation
- Search indexing

## Monitoring & Regression Detection

### CI Performance Checks

Currently: Manual checks in development
Future: Automated regression detection in CI

### Local Performance Testing

```bash
# Before changes
git checkout main
cargo bench -- baseline

# After changes
git checkout feature-branch
cargo bench -- comparison

# Compare results
```

## Resources

- **Profiling Guide**: https://nnethercote.github.io/perf-book/
- **Cargo Book - Performance**: https://doc.rust-lang.org/cargo/reference/profiles.html
- **Benchmarking**: https://doc.rust-lang.org/cargo/commands/cargo-bench.html
