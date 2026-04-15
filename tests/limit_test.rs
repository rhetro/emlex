// Increase recursion limit if testing extremely deep ASTs
#![recursion_limit = "256"]

use emlex::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extreme_recursion_depth() {
        let x: f64 = 2.0;

        // Deeply nested AST structure to stress-test the TT-muncher recursion.
        // It recursively applies `exp` 10 times: exp(exp(exp...))
        let (_, result) = eml!(
            (eml (eml (eml (eml (eml (eml (eml (eml (eml (eml x 1) 1) 1) 1) 1) 1) 1) 1) 1) 1)
        );

        let expected = x
            .exp()
            .exp()
            .exp()
            .exp()
            .exp()
            .exp()
            .exp()
            .exp()
            .exp()
            .exp();

        assert_eq!(result, expected);
    }
}
