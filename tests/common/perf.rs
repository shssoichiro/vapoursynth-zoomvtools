use anyhow::Result;
use std::time::{Duration, Instant};
use vapoursynth::prelude::Node;

#[derive(Debug)]
pub struct PerfResult {
    pub filter_name: String,
    pub implementation: String,
    pub total_time: Duration,
    pub frames_processed: usize,
    pub time_per_frame: Duration,
}

impl PerfResult {
    pub fn fps(&self) -> f64 {
        self.frames_processed as f64 / self.total_time.as_secs_f64()
    }
}

/// Measure filter performance by processing frames
pub fn measure_filter_performance(
    node: &Node,
    num_frames: usize,
    implementation: &str,
    filter_name: &str,
) -> Result<PerfResult> {
    let start = Instant::now();

    for n in 0..num_frames {
        let _ = node.get_frame(n)?;
    }

    let total_time = start.elapsed();

    Ok(PerfResult {
        filter_name: filter_name.to_string(),
        implementation: implementation.to_string(),
        total_time,
        frames_processed: num_frames,
        time_per_frame: total_time / num_frames as u32,
    })
}

/// Compare performance between C and Rust implementations
pub fn compare_performance(c_result: &PerfResult, r_result: &PerfResult) -> String {
    let speedup = c_result.total_time.as_secs_f64() / r_result.total_time.as_secs_f64();

    format!(
        "Performance comparison for {}:\n\
         C MVTools:  {:.2} fps ({:.2}ms/frame)\n\
         Rust zoomv: {:.2} fps ({:.2}ms/frame)\n\
         Speedup: {:.2}x {}",
        c_result.filter_name,
        c_result.fps(),
        c_result.time_per_frame.as_secs_f64() * 1000.0,
        r_result.fps(),
        r_result.time_per_frame.as_secs_f64() * 1000.0,
        speedup,
        if speedup > 1.0 {
            "(Rust faster)"
        } else {
            "(C faster)"
        }
    )
}
