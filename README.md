# Cuckatoo Rust Miner

A Rust implementation of the Cuckatoo mining algorithm, converted from the original C++ reference implementation.

## Project Structure

This is a Cargo workspace with two crates:

- **`cuckatoo-core`**: Core algorithms and data types
  - SipHash-2-4 implementation for edge generation
  - Lean trimming algorithm
  - Cycle verification for 42-cycles
  - Bitmap data structures for efficient operations

- **`cuckatoo-miner`**: CLI application for mining
  - Command-line interface with clap
  - Support for different mining modes
  - Tuning mode for performance testing

## Features

### Milestone 1 - CPU "Lean" Baseline ✅

- ✅ Cargo workspace with `cuckatoo-core` and `cuckatoo-miner` crates
- ✅ Header→edge generation using SipHash-2-4
- ✅ Lean edge bitmap and node degree bitmap implementation
- ✅ 42-round trim loop for edge reduction
- ✅ Cycle verifier for 42-cycle detection
- ✅ CLI with parity to README wording:
  - `--edge-bits` parameter (10-32)
  - `--mode lean` (with support for mean/slean)
  - `--tuning` for offline mode
  - Timing output showing Searching time vs Trimming time

## Building

### Prerequisites

- Rust 1.70+ (edition 2021)
- C compiler (gcc/clang) for native dependencies

### Build Commands

```bash
# Check if everything compiles
cargo check

# Run tests
cargo test

# Build in release mode
cargo build --release

# Run the miner
cargo run --bin cuckatoo-miner -- --tuning --edge-bits 12 --mode lean
```

## Usage

### Basic Usage

```bash
# Run with default settings (edge-bits=12, mode=lean)
cargo run --bin cuckatoo-miner

# Run in tuning mode with custom edge bits
cargo run --bin cuckatoo-miner -- --tuning --edge-bits 16 --mode lean

# Show help
cargo run --bin cuckatoo-miner -- --help
```

### Command Line Options

- `--edge-bits, -e`: Number of edge bits (10-32, default: 12)
- `--mode, -m`: Mining mode (lean/mean/slean, default: lean)
- `--tuning, -t`: Enable tuning mode (offline, no stratum connection)

### Example Output

```
Cuckatoo Rust Miner
Edge bits: 12
Mode: lean
Tuning: true

Number of edges: 4096
Generating edges...
Generated 4096 edges in 1.08318ms
Performing lean trimming...
Trimmed to 4095 edges in 205.835371ms
Searching for cycles...
Found 0 solutions in 11.964416ms

Timing Summary:
  Trimming time: 205.835371ms
  Searching time: 11.964416ms
  Total time: 217.799787ms

No solutions found.
```

## Algorithm Overview

### SipHash-2-4
- Generates edges from block headers and nonces
- Uses 4 SipHash keys for deterministic edge generation
- Applies node mask based on edge bits

### Lean Trimming
- Removes edges connected to degree-1 nodes
- Performs 42 rounds of trimming
- Uses bitmaps for efficient edge tracking
- Reduces graph size while preserving cycles

### Cycle Verification
- Searches for 42-cycles in the trimmed graph
- Uses bipartite graph traversal
- Alternates between U and V partitions
- Backtracking algorithm for cycle detection

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_siphash_basic

# Run tests with output
cargo test -- --nocapture
```

### Integration Tests

The CLI can be tested with various parameters:

```bash
# Test small graphs
cargo run --bin cuckatoo-miner -- --tuning --edge-bits 10 --mode lean

# Test larger graphs
cargo run --bin cuckatoo-miner -- --tuning --edge-bits 16 --mode lean

# Test different modes
cargo run --bin cuckatoo-miner -- --tuning --edge-bits 12 --mode mean
```

## Performance

Current performance characteristics (on typical hardware):

- **Edge Generation**: ~1ms for 4096 edges (edge-bits=12)
- **Lean Trimming**: ~200ms for 4096 edges (42 rounds)
- **Cycle Search**: ~12ms for trimmed graph
- **Memory Usage**: Linear with number of edges

## Future Milestones

- **Milestone 2**: GPU acceleration with OpenCL/Metal
- **Milestone 3**: Stratum protocol integration
- **Milestone 4**: Advanced trimming algorithms (mean, slean)
- **Milestone 5**: Performance optimizations and benchmarking

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `cargo test` to ensure everything works
6. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

This implementation is based on the original C++ Cuckatoo Reference Miner by the MimbleWimble Coin development team.
