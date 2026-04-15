use emlex::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_values() {
        // Mathematical edge case: ln(0) should be -Infinity
        let zero: f64 = 0.0;
        let (_, ln_result) = eml!((eml 1 (eml (eml 1 zero) 1)));
        assert_eq!(ln_result, std::f64::NEG_INFINITY);

        // exp(0) should be 1.0
        let (_, exp_result) = eml!((eml zero 1));
        assert_eq!(exp_result, 1.0);
    }

    #[test]
    fn test_negative_values() {
        // Mathematical edge case: ln of negative number is NaN
        let neg: f64 = -5.0;
        let (_, ln_result) = eml!((eml 1 (eml (eml 1 neg) 1)));
        assert!(ln_result.is_nan());
    }

    #[test]
    fn test_infinity() {
        // Math with Infinity
        let inf: f64 = std::f64::INFINITY;
        let (_, result) = eml!((eml inf 1));
        assert_eq!(result, std::f64::INFINITY);
    }
}
