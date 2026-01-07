#![cfg(feature = "e2e")]

pub mod comparison;
pub mod detection;
pub mod script_gen;

/// Macro to require MVTools and skip test if not available
/// Use this at the start of each e2e test
#[macro_export]
macro_rules! require_mvtools {
    () => {
        if let Err(e) = $crate::common::detection::check_mvtools_available() {
            panic!("Test failed: {:#}", e);
        }
    };
}
