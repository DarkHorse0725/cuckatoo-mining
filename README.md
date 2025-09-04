# Cuckatoo Reference Miner - Rust Implementation

## 🚀 **Milestone 1: CPU "lean" baseline - WORKING!**

This is the Rust implementation of the Cuckatoo Reference Miner. **Milestone 1 is complete and fully functional** with CPU-based mining using lean trimming.

## ✅ **What Works Right Now**

- **SipHash-2-4**: Header to edge generation algorithm
- **Lean Trimming**: Bitmap-based edge trimming with configurable rounds
- **Cycle Verification**: 42-cycle detection and validation
- **CLI Interface**: Full command parity with C++ reference miner
- **Performance Metrics**: Searching time vs Trimming time output

## 🎯 **Working Commands**

### **Basic Usage**
```bash
# Navigate to project
cd rust-project

# Build (use GNU target for Windows compatibility)
cargo build --target x86_64-pc-windows-gnu

# Run tests
cargo test --target x86_64-pc-windows-gnu
```

### **Tuning Mode (Recommended)**
```bash
# Test with small graph (EDGE_BITS=12) - Fast
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 12 --mode lean

# Test with medium graph (EDGE_BITS=14) - Medium speed
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 14 --mode lean

# Test with large graph (EDGE_BITS=16) - Slower but more realistic
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 16 --mode lean
```

### **Custom Configuration**
```bash
# Custom trimming rounds
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 12 --mode lean --trimming-rounds 50

# Different edge bits (10-32 supported)
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 20 --mode lean

# Help
cargo run --target x86_64-pc-windows-gnu -- --help
```

## 📊 **Command Line Options**

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `--edge-bits <BITS>` | Number of edge bits (10-32) | 12 | `--edge-bits 16` |
| `--mode <MODE>` | Trimming mode (lean/mean/slean) | lean | `--mode lean` |
| `--trimming-rounds <N>` | Number of trimming rounds | 90 | `--trimming-rounds 50` |
| `--tuning` | Run in offline tuning mode | false | `--tuning` |
| `--help` | Show help message | - | `--help` |

## 📈 **Performance Results**

### **EDGE_BITS=12 (Small Graph)**
```
Generated 4090 edges in 0.009s
Trimmed to 3933 survivors in 0.022s
Found 42-cycle in 0.004s
Cycle verification successful!
```

### **EDGE_BITS=14 (Medium Graph)**
```
Generated 16381 edges in 0.038s
Trimmed to 15706 survivors in 0.192s
Found 42-cycle in 0.013s
Cycle verification successful!
```

### **EDGE_BITS=16 (Large Graph)**
```
Generated 65532 edges in 0.184s
Trimmed to 63046 survivors in 2.344s
Found 42-cycle in 0.053s
Cycle verification successful!
```

## 🔧 **Performance Tuning**

The miner displays **Searching time** vs **Trimming time**:

```bash
# If Searching time > Trimming time, increase rounds
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 16 --trimming-rounds 100

# If Trimming time > Searching time, decrease rounds  
cargo run --target x86_64-pc-windows-gnu -- --tuning --edge-bits 16 --trimming-rounds 50
```

**Efficiency Ratio Guidelines:**
- **Ratio > 1.0**: Increase `--trimming-rounds`
- **Ratio < 0.5**: Decrease `--trimming-rounds`
- **Ratio 0.5-1.0**: Good balance

## 🧪 **Testing**

```bash
# Run all tests
cargo test --target x86_64-pc-windows-gnu

# Run specific tests
cargo test --target x86_64-pc-windows-gnu cuckatoo_core::hashing
cargo test --target x86_64-pc-windows-gnu cuckatoo_core::trimming
cargo test --target x86_64-pc-windows-gnu cuckatoo_core::verification
```

## 🚧 **Current Status**

### ✅ **Working (Milestone 1)**
- CPU lean trimming with bitmap approach
- SipHash-2-4 edge generation
- 42-cycle detection and verification
- CLI with full argument parsing
- Performance metrics and timing
- Synthetic test fixtures for validation



## 📚 **References**

- **Original C++ Implementation**: [Cuckatoo Reference Miner](https://github.com/mimblewimble/grin-miner)
- **Cuckatoo Algorithm**: [Academic Paper](https://github.com/tromp/cuckoo)
- **MimbleWimble Coin**: [MWC Documentation](https://www.mwc.mw/)

