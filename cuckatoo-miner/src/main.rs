//! Cuckatoo Miner CLI Runner
//! 
//! This implements the CLI interface for the Cuckatoo Reference Miner
//! with parity to the C++ version as specified in Milestone 1.

use cuckatoo_core::{
    Config, TrimmingMode, CycleVerifier, LeanTrimmer, 
    hashing::{generate_edges, SipHash}
};
use std::time::Instant;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cuckatoo Reference Miner v0.1.0 (Rust)");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;
    
    println!("Configuration: EDGE_BITS={}, mode={}, rounds={}, tuning={}", 
             config.edge_bits, config.mode, config.trimming_rounds, config.tuning);
    
    // Validate configuration
    config.validate()?;
    
    // Test header (simple test data for tuning mode)
    let header = [0u8; 80];
    let nonce = 0u64;
    
    // Generate edges using SipHash-2-4
    println!("Generating edges...");
    let start_time = Instant::now();
    let edges = generate_edges(&header, nonce, config.edge_bits)?;
    let generation_time = start_time.elapsed();
    
    println!("Generated {} edges in {:.6}s", edges.len(), generation_time.as_secs_f64());
    
    // Perform lean trimming
    println!("Performing lean trimming...");
    let trim_start = Instant::now();
    let mut trimmer = LeanTrimmer::with_rounds(config.edge_bits, config.trimming_rounds);
    let survivors = trimmer.trim_edges(&edges, config.trimming_rounds)?;
    let trim_time = trim_start.elapsed();
    
    println!("Trimmed to {} survivors in {:.6}s", survivors.len(), trim_time.as_secs_f64());
    
    // Print timing information as specified in requirements
    println!("Searching time: {:.6}s", generation_time.as_secs_f64());
    println!("Trimming time: {:.6}s", trim_time.as_secs_f64());
    
    // Test cycle verification if we have survivors
    if !survivors.is_empty() {
        println!("Testing cycle verification...");
        let verify_start = Instant::now();
        let mut verifier = CycleVerifier::new();
        let cycle = verifier.find_42_cycle(&survivors)?;
        let verify_time = verify_start.elapsed();
        
        match cycle {
            Some(cycle) => {
                println!("Found 42-cycle in {:.6}s", verify_time.as_secs_f64());
                println!("Cycle length: {}", cycle.len());
                
                // Print SipHash keys for verification
                let siphash = SipHash::new();
                let keys = siphash.get_key();
                println!("SipHash keys: [0x{:016x}, 0x{:016x}]", keys[0], keys[1]);
                
                // Print cycle edge indices
                println!("Cycle edge indices:");
                for (i, edge) in cycle.windows(2).enumerate() {
                    if let [node1, node2] = edge {
                        // Find the edge index in the original edges list
                        if let Some(edge_index) = edges.iter().position(|&e| 
                            (e.u == *node1 && e.v == *node2) || (e.u == *node2 && e.v == *node1)
                        ) {
                            println!("  Edge {}: {} -> {} (index: {})", i, node1, node2, edge_index);
                        }
                    }
                }
                // Handle the last edge (cycle[last] -> cycle[0])
                if cycle.len() >= 2 {
                    let last_node = cycle[cycle.len() - 1];
                    let first_node = cycle[0];
                    if let Some(edge_index) = edges.iter().position(|&e| 
                        (e.u == last_node && e.v == first_node) || (e.u == first_node && e.v == last_node)
                    ) {
                        println!("  Edge {}: {} -> {} (index: {})", cycle.len() - 1, last_node, first_node, edge_index);
                    }
                }
                
                // Verify the cycle is valid
                if verifier.verify_specific_cycle(&cycle, &survivors) {
                    println!("Cycle verification successful!");
                } else {
                    println!("Cycle verification failed!");
                }
            },
            None => println!("No 42-cycle found in {:.6}s", verify_time.as_secs_f64()),
        }
        
        let metrics = verifier.metrics();
        println!("Performance metrics: solutions_found={}, searching_time={:.6}s", 
                 metrics.solutions_found, metrics.searching_time);
    }
    
    if config.tuning {
        println!("Tuning test completed successfully!");
    } else {
        println!("Mining completed!");
    }
    
    Ok(())
}

/// Parse command line arguments
fn parse_args(args: &[String]) -> Result<Config, Box<dyn std::error::Error>> {
    let mut edge_bits = 12; // Default to small edge bits for testing
    let mut mode = TrimmingMode::Lean;
    let mut trimming_rounds = 90;
    let mut tuning = false;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--edge-bits" => {
                i += 1;
                if i < args.len() {
                    edge_bits = args[i].parse()?;
                } else {
                    return Err("Missing value for --edge-bits".into());
                }
            },
            "--mode" => {
                i += 1;
                if i < args.len() {
                    mode = args[i].parse()?;
                } else {
                    return Err("Missing value for --mode".into());
                }
            },
            "--tuning" => {
                tuning = true;
            },
            "--trimming-rounds" => {
                i += 1;
                if i < args.len() {
                    trimming_rounds = args[i].parse()?;
                } else {
                    return Err("Missing value for --trimming-rounds".into());
                }
            },
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            },
            _ => {
                if args[i].starts_with('-') {
                    return Err(format!("Unknown option: {}", args[i]).into());
                }
            }
        }
        i += 1;
    }
    
    Ok(Config {
        edge_bits,
        trimming_rounds,
        mode,
        tuning,
    })
}

/// Print usage information
fn print_usage() {
    println!("Cuckatoo Reference Miner v0.1.0 (Rust)");
    println!();
    println!("Usage: cuckatoo-miner [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --edge-bits <BITS>     Number of edge bits (10-32, default: 12)");
    println!("  --mode <MODE>          Trimming mode: lean, mean, slean (default: lean)");
    println!("  --trimming-rounds <N>  Number of trimming rounds (default: 90)");
    println!("  --tuning               Run in tuning mode (offline)");
    println!("  --help, -h             Show this help message");
    println!();
    println!("Examples:");
    println!("  cuckatoo-miner --tuning --edge-bits 12 --mode lean");
    println!("  cuckatoo-miner --edge-bits 16 --mode lean");
}
