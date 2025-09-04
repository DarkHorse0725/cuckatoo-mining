//! Timing and performance measurement for Cuckatoo
//! 
//! This module provides utilities for measuring performance and
//! benchmarking different components of the Cuckatoo miner.

use crate::{PerformanceMetrics, Result, CuckatooError};
use std::time::{Instant, Duration};
use std::collections::HashMap;

/// Performance timer for measuring execution time
pub struct PerformanceTimer {
    /// Start time
    start_time: Instant,
    /// Checkpoints for measuring different phases
    checkpoints: HashMap<String, Instant>,
    /// Total metrics
    metrics: PerformanceMetrics,
}

impl PerformanceTimer {
    /// Create a new performance timer
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: HashMap::new(),
            metrics: PerformanceMetrics::new(),
        }
    }
    
    /// Start timing a specific phase
    pub fn start_phase(&mut self, phase: &str) {
        self.checkpoints.insert(phase.to_string(), Instant::now());
    }
    
    /// End timing a specific phase
    pub fn end_phase(&mut self, phase: &str) -> Result<Duration> {
        if let Some(start_time) = self.checkpoints.get(phase) {
            let duration = start_time.elapsed();
            println!("Phase '{}' completed in {:?}", phase, duration);
            Ok(duration)
        } else {
            Err(CuckatooError::InternalError(
                format!("Phase '{}' was not started", phase)
            ))
        }
    }
    
    /// Get duration for a specific phase
    pub fn get_phase_duration(&self, phase: &str) -> Option<Duration> {
        self.checkpoints.get(phase).map(|start| start.elapsed())
    }
    
    /// Get total elapsed time
    pub fn total_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Set searching time
    pub fn set_searching_time(&mut self, duration: Duration) {
        self.metrics.searching_time = duration.as_secs_f64();
    }
    
    /// Set trimming time
    pub fn set_trimming_time(&mut self, duration: Duration) {
        self.metrics.trimming_time = duration.as_secs_f64();
    }
    
    /// Set graphs processed
    pub fn set_graphs_processed(&mut self, count: u64) {
        self.metrics.graphs_processed = count;
    }
    
    /// Set solutions found
    pub fn set_solutions_found(&mut self, count: u64) {
        self.metrics.solutions_found = count;
    }
    
    /// Calculate mining rate
    pub fn calculate_mining_rate(&mut self) {
        let total_time = self.metrics.total_time();
        if total_time > 0.0 {
            self.metrics.mining_rate = self.metrics.graphs_processed as f64 / total_time;
        }
    }
    
    /// Get performance metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    /// Get mutable performance metrics
    pub fn metrics_mut(&mut self) -> &mut PerformanceMetrics {
        &mut self.metrics
    }
    
    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.checkpoints.clear();
        self.metrics = PerformanceMetrics::new();
    }
}

impl Default for PerformanceTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark runner for comparing different implementations
pub struct BenchmarkRunner {
    /// Benchmark results
    results: HashMap<String, BenchmarkResult>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }
    
    /// Run a benchmark
    pub fn run_benchmark<F, R>(
        &mut self,
        name: &str,
        iterations: usize,
        benchmark_fn: F,
    ) -> BenchmarkResult
    where
        F: Fn() -> R,
    {
        let mut times = Vec::with_capacity(iterations);
        let mut total_time = Duration::ZERO;
        
        // Warm up
        for _ in 0..iterations / 10 {
            benchmark_fn();
        }
        
        // Run benchmark
        for _ in 0..iterations {
            let start = Instant::now();
            benchmark_fn();
            let duration = start.elapsed();
            times.push(duration);
            total_time += duration;
        }
        
        // Calculate statistics
        times.sort();
        let min_time = times[0];
        let max_time = times[iterations - 1];
        let avg_time = total_time / iterations as u32;
        let median_time = times[iterations / 2];
        
        let result = BenchmarkResult {
            name: name.to_string(),
            iterations,
            min_time,
            max_time,
            avg_time,
            median_time,
            total_time,
        };
        
        self.results.insert(name.to_string(), result.clone());
        result
    }
    
    /// Compare two benchmarks
    pub fn compare(&self, name1: &str, name2: &str) -> Option<BenchmarkComparison> {
        let result1 = self.results.get(name1)?;
        let result2 = self.results.get(name2)?;
        
        let speedup = result1.avg_time.as_secs_f64() / result2.avg_time.as_secs_f64();
        let improvement = if speedup > 1.0 {
            format!("{:.2}x faster", speedup)
        } else {
            format!("{:.2}x slower", 1.0 / speedup)
        };
        
        Some(BenchmarkComparison {
            baseline: result1.clone(),
            comparison: result2.clone(),
            speedup,
            improvement,
        })
    }
    
    /// Print all benchmark results
    pub fn print_results(&self) {
        println!("\n=== Benchmark Results ===");
        for (name, result) in &self.results {
            println!("{}:", name);
            println!("  Iterations: {}", result.iterations);
            println!("  Average: {:?}", result.avg_time);
            println!("  Median: {:?}", result.median_time);
            println!("  Min: {:?}", result.min_time);
            println!("  Max: {:?}", result.max_time);
            println!("  Total: {:?}", result.total_time);
            println!();
        }
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Number of iterations
    pub iterations: usize,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
    /// Average execution time
    pub avg_time: Duration,
    /// Median execution time
    pub median_time: Duration,
    /// Total execution time
    pub total_time: Duration,
}

/// Comparison between two benchmarks
#[derive(Debug)]
pub struct BenchmarkComparison {
    /// Baseline benchmark
    pub baseline: BenchmarkResult,
    /// Comparison benchmark
    pub comparison: BenchmarkResult,
    /// Speedup ratio
    pub speedup: f64,
    /// Human-readable improvement description
    pub improvement: String,
}

/// Utility for measuring execution time of a function
pub fn measure_time<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Utility for measuring execution time with logging
pub fn measure_time_logged<F, R>(name: &str, f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    println!("Starting {}", name);
    let (result, duration) = measure_time(f);
    println!("{} completed in {:?}", name, duration);
    (result, duration)
}

/// Utility for measuring execution time and updating metrics
pub fn measure_time_with_metrics<F, R>(
    timer: &mut PerformanceTimer,
    phase: &str,
    f: F,
) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    timer.start_phase(phase);
    let (result, duration) = measure_time(f);
    timer.end_phase(phase).ok(); // Ignore errors for now
    (result, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_performance_timer() {
        let mut timer = PerformanceTimer::new();
        
        timer.start_phase("test");
        thread::sleep(Duration::from_millis(10));
        let duration = timer.end_phase("test").unwrap();
        
        assert!(duration >= Duration::from_millis(10));
        assert_eq!(timer.get_phase_duration("test"), Some(duration));
    }
    
    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new();
        
        let result = runner.run_benchmark("test", 5, || {
            thread::sleep(Duration::from_millis(1));
        });
        
        assert_eq!(result.name, "test");
        assert_eq!(result.iterations, 5);
        assert!(result.avg_time >= Duration::from_millis(1));
    }
    
    #[test]
    fn test_measure_time() {
        let (_, duration) = measure_time(|| {
            thread::sleep(Duration::from_millis(5));
        });
        
        assert!(duration >= Duration::from_millis(5));
    }
    
    #[test]
    fn test_measure_time_logged() {
        let (_, duration) = measure_time_logged("test", || {
            thread::sleep(Duration::from_millis(5));
        });
        
        assert!(duration >= Duration::from_millis(5));
    }
}
