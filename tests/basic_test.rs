use emlex::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eml_primitive_exp() {
        let x: f64 = 2.0;
        // exp(x) -> eml(x, 1)
        let (ast, val) = eml!((eml x 1.0));

        assert_eq!(ast.to_eml(), "(eml x 1)");
        // Mathematical evaluation: exp(x) - ln(1) = exp(x) - 0 = exp(x)
        assert_eq!(val, x.exp());
    }

    #[test]
    fn test_eml_primitive_ln() {
        let x: f64 = 10.0;
        // ln(x) -> eml(1, eml(eml(1, x), 1))
        let (ast, val) = eml!((eml 1.0 (eml (eml 1.0 x) 1.0)));

        assert_eq!(ast.to_eml(), "(eml 1 (eml (eml 1 x) 1))");

        // Compare accounting for floating-point calculation errors
        let expected = x.ln();
        assert!((val - expected).abs() < 1e-10);
    }

    #[test]
    fn test_eml_primitive_sub() {
        let x: f64 = 2.0;
        let y: f64 = 3.0;
        // eml(x, y) = exp(x) - ln(y)
        let (ast, val) = eml!((eml x y));

        assert_eq!(ast.to_eml(), "(eml x y)");
        assert_eq!(val, x.exp() - y.ln());
    }

    #[test]
    fn test_eml_literal_injection() {
        // Direct numeric literals
        let (ast, val) = eml!((eml 2.0 3.0));

        assert_eq!(ast.to_eml(), "(eml 2 3)");
        assert_eq!(val, 2.0_f64.exp() - 3.0_f64.ln());
    }

    #[test]
    fn test_eml_deep_tree() {
        let a: f64 = 2.0;
        let b: f64 = 3.0;

        let (ast, val) = eml!((eml (eml a b) 1.0));

        assert_eq!(ast.to_eml(), "(eml (eml a b) 1)");
        assert_eq!(val, (a.exp() - b.ln()).exp());
    }

    // ========================================================================
    // AST Optimization Tests
    // ========================================================================

    #[test]
    fn test_eml_ast_optimization() {
        let z: f64 = 5.0;

        // 1. exp(ln(z)) -> z
        // Structure: (eml (eml 1 (eml (eml 1 z) 1)) 1)
        let (ast_exp_ln, _) = eml!((eml (eml 1.0 (eml (eml 1.0 z) 1.0)) 1.0));
        assert_eq!(ast_exp_ln.to_eml(), "(eml (eml 1 (eml (eml 1 z) 1)) 1)");

        // After the optimization pass, it successfully extracts just 'z'
        let optimized_1 = ast_exp_ln.optimize();
        assert_eq!(optimized_1.to_eml(), "z");
        assert_eq!(optimized_1, EExpr::Var("z"));

        // 2. ln(exp(z)) -> z
        // Structure: (eml 1 (eml (eml 1 (eml z 1)) 1))
        let (ast_ln_exp, _) = eml!((eml 1.0 (eml (eml 1.0 (eml z 1.0)) 1.0)));
        assert_eq!(ast_ln_exp.to_eml(), "(eml 1 (eml (eml 1 (eml z 1)) 1))");

        // This pattern also cancels out to 'z' in the same way
        let optimized_2 = ast_ln_exp.optimize();
        assert_eq!(optimized_2.to_eml(), "z");
        assert_eq!(optimized_2, EExpr::Var("z"));
    }
}
