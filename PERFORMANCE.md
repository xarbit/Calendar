# Performance Optimization Guide

## Current Optimizations

### Calendar State Caching
- `CalendarState` caches month/week calculations to avoid recomputing on every render
- Cache is automatically invalidated when month/year changes
- Reduces CPU usage when switching views or interacting with the UI

### Build Optimizations
- Always use `cargo build --release` for production builds
- Release mode enables Rust compiler optimizations (3-10x faster than debug)
- LTO (Link Time Optimization) can be enabled in `Cargo.toml` for further gains

## Known Performance Considerations

### Widget Tree Rebuilding
The entire widget tree is rebuilt on every render. This is a limitation of the current iced/libcosmic architecture. The framework is optimized for this pattern, so it's generally not an issue.

### Responsive UI Behavior
- Window resizing triggers immediate re-layout
- Sidebar toggle is instant using libcosmic's built-in responsive system
- View switching (Month/Week/Day) is fast with caching

### Future Optimizations

1. **Lazy Rendering**
   - Only render visible calendar cells
   - Virtualize long event lists

2. **Event Data Caching**
   - Cache CalDAV responses
   - Index events by date for O(1) lookups

3. **Async Operations**
   - Move CalDAV sync to background task
   - Non-blocking event loading

4. **Widget Reuse**
   - Implement widget pooling for day cells
   - Reuse common components

## Profiling

To profile the application:

```bash
# CPU profiling with perf (Linux)
perf record --call-graph=dwarf cargo run --release
perf report

# Memory profiling with heaptrack (Linux)
heaptrack cargo run --release

# Flamegraph generation
cargo flamegraph
```

## Performance Tips

1. **Always use release mode** for testing performance
2. **Minimize state changes** - only update what's necessary
3. **Cache computed values** - use `CalendarState` pattern for expensive calculations
4. **Batch updates** - combine multiple state changes into one update
5. **Profile before optimizing** - measure to find real bottlenecks

## Benchmarking Results

Run benchmarks with:
```bash
cargo bench
```

(Benchmarks to be added in future releases)
