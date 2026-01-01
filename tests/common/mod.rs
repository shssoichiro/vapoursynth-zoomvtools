pub mod comparison;
pub mod detection;
pub mod perf;
pub mod script_gen;

// Re-export commonly used items
pub use comparison::{
    ComparisonConfig, PixelDiff, PixelDifference, assert_frames_match, compare_frame_properties,
    compare_frames, compare_motion_vectors,
};
pub use detection::check_mvtools_available;
pub use perf::{PerfResult, compare_performance, measure_filter_performance};
pub use script_gen::{ClipContentType, FilterParams, TestClipConfig, generate_comparison_script};

/// Macro to require MVTools and skip test if not available
/// Use this at the start of each e2e test
#[macro_export]
macro_rules! require_mvtools {
    () => {
        if let Err(e) = $crate::common::check_mvtools_available() {
            panic!("Test failed: {:#}", e);
        }
    };
}
