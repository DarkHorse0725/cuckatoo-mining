use clap::{Parser, ValueEnum};
use cuckatoo_core::{
    constants::*,
    types::{MinerConfig, MiningMode},
    SipHash, LeanTrimmer, CycleVerifier,
};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "cuckatoo-miner")]
#[command(about = "Rust implementation of Cuckatoo mining algorithm")]
struct Args {
    /// Number of edge bits (10-32)
    #[arg(long, short, default_value = "12")]
    edge_bits: u32,

    /// Mining mode
    #[arg(long, short, value_enum, default_value = "lean")]
    mode: MiningModeArg,

    /// Enable tuning mode (offline)
    #[arg(long, short)]
    tuning: bool,
}

#[derive(ValueEnum, Clone, Copy)]
enum MiningModeArg {
    Lean,
    Mean,
    Slean,
}

impl std::fmt::Display for MiningModeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiningModeArg::Lean => write!(f, "lean"),
            MiningModeArg::Mean => write!(f, "mean"),
            MiningModeArg::Slean => write!(f, "slean"),
        }
    }
}

impl From<MiningModeArg> for MiningMode {
    fn from(mode: MiningModeArg) -> Self {
        match mode {
            MiningModeArg::Lean => MiningMode::Lean,
            MiningModeArg::Mean => MiningMode::Mean,
            MiningModeArg::Slean => MiningMode::Slean,
        }
    }
}

fn main() {
    let args = Args::parse();

    // Validate edge bits
    if !validate_edge_bits(args.edge_bits) {
        eprintln!("Error: edge-bits must be between {} and {}", MIN_EDGE_BITS, MAX_EDGE_BITS);
        std::process::exit(1);
    }

    println!("Cuckatoo Rust Miner");
    println!("Edge bits: {}", args.edge_bits);
    println!("Mode: {}", args.mode);
    println!("Tuning: {}", args.tuning);
    println!();

    // Create configuration
    let config = MinerConfig {
        edge_bits: args.edge_bits,
        mode: args.mode.into(),
        tuning: args.tuning,
    };

    // Run the miner
    if let Err(e) = run_miner(config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_miner(config: MinerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let number_of_edges = number_of_edges(config.edge_bits);
    println!("Number of edges: {}", number_of_edges);

    // Create SipHash instance with default keys
    let siphash = SipHash::default();

    // Generate test header (in real implementation, this would come from stratum)
    let header = b"test header for mining";
    
    // Generate nonces for testing
    let nonces: Vec<u64> = (0..number_of_edges).collect();

    // Generate edges
    println!("Generating edges...");
    let trimming_start = Instant::now();
    let edges = siphash.generate_edges(header, &nonces, config.edge_bits);
    let trimming_time = trimming_start.elapsed();
    println!("Generated {} edges in {:?}", edges.len(), trimming_time);

    // Perform trimming
    println!("Performing lean trimming...");
    let trimming_start = Instant::now();
    let trimmer = LeanTrimmer::new(config.edge_bits);
    let trimmed_edges = trimmer.trim_edges(&edges);
    let trimming_time = trimming_start.elapsed();
    println!("Trimmed to {} edges in {:?}", trimmed_edges.len(), trimming_time);

    // Search for cycles
    println!("Searching for cycles...");
    let searching_start = Instant::now();
    let verifier = CycleVerifier::new(config.edge_bits);
    let solutions = verifier.find_cycles(&trimmed_edges);
    let searching_time = searching_start.elapsed();
    println!("Found {} solutions in {:?}", solutions.len(), searching_time);

    // Print timing information
    println!();
    println!("Timing Summary:");
    println!("  Trimming time: {:?}", trimming_time);
    println!("  Searching time: {:?}", searching_time);
    println!("  Total time: {:?}", trimming_time + searching_time);

    // Print solutions if found
    if !solutions.is_empty() {
        println!();
        println!("Found {} solutions:", solutions.len());
        for (i, solution) in solutions.iter().enumerate() {
            println!("  Solution {}: {:?}", i + 1, solution.edges);
        }
    } else {
        println!();
        println!("No solutions found.");
    }

    Ok(())
}
