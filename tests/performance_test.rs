// Performance test for zero‑cost abstraction.
// Run this file with:
//
//     cargo test --release --test performance_test -- --nocapture
//
// (Release mode is required for accurate macro vs native timing.)
//

use emlex::prelude::*;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_cost_abstraction_runtime() {
        let iterations = 10_000_000;
        let mut dummy_sum_macro = 0.0;
        let mut dummy_sum_native = 0.0;
        let val: f64 = 1.0000001;

        let start_macro = Instant::now();
        for _ in 0..iterations {
            let (_, res) = eml!((eml (eml val val) (eml val val)));
            dummy_sum_macro += res;
        }
        let macro_duration = start_macro.elapsed();

        let start_native = Instant::now();
        for _ in 0..iterations {
            let res = (val.exp() - val.ln()).exp() - (val.exp() - val.ln()).ln();
            dummy_sum_native += res;
        }
        let native_duration = start_native.elapsed();

        println!("[Performance Test]");
        println!("Macro execution time : {:?}", macro_duration);
        println!("Native execution time: {:?}", native_duration);

        assert_eq!(dummy_sum_macro, dummy_sum_native);

        #[cfg(debug_assertions)]
        println!(
            "Note: Debug build detected. Run with `--release` for accurate zero-cost measurements."
        );

        #[cfg(not(debug_assertions))]
        assert!(
            macro_duration.as_millis() <= native_duration.as_millis() + 50,
            "Macro is slower than native in release mode!"
        );
    }
}
