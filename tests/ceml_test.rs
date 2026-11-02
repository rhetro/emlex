use emlex::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceml_complex_literals() {
        let (ast_i, val_i) = ceml!((i));
        assert_eq!(ast_i, CExpr::I);
        assert_eq!(ast_i.to_ceml(), "(i)");
        assert_eq!(val_i, Complex::new(0.0, 1.0));

        let (ast_c, val_c) = ceml!((complex 3.0 4.0));
        assert_eq!(ast_c, CExpr::ComplexLit(3.0, 4.0));
        assert_eq!(ast_c.to_ceml(), "(complex 3 4)");
        assert_eq!(val_c, Complex::new(3.0, 4.0));

        let (ast_lit, val_lit) = ceml!(42.0);
        assert_eq!(ast_lit, CExpr::Lit(42.0));
        assert_eq!(ast_lit.to_ceml(), "42");
        assert_eq!(val_lit, Complex::new(42.0, 0.0));
    }

    #[test]
    fn test_ceml_math_operations() {
        let z1 = Complex::new(1.0, 2.0);

        let (ast, val) = ceml!((add z1 (i)));

        assert_eq!(
            ast,
            CExpr::Add(Box::new(CExpr::Var("z1")), Box::new(CExpr::I))
        );
        assert_eq!(ast.to_ceml(), "(add z1 (i))");
        assert_eq!(val, z1 + Complex::new(0.0, 1.0));
    }

    #[test]
    fn test_ceml_deep_tree_with_reverse_dsl() {
        let pi = std::f64::consts::PI;

        let (ast, val) = ceml!(
            (add (exp (mul (i) pi)) 1.0)
        );

        let expected_dsl = "(add (exp (mul (i) pi)) 1)";
        assert_eq!(ast.to_ceml(), expected_dsl);

        assert!(val.re.abs() < 1e-15);
        assert!(val.im.abs() < 1e-15);
    }

    // ========================================================================
    // NEW TESTS FOR LANGUAGE COMPLETENESS
    // ========================================================================

    #[test]
    fn test_ceml_exhaustive_variants() {
        // Ensure Sub and Div AST variants are generated and evaluated correctly
        let z1 = Complex::new(10.0, 0.0);
        let z2 = Complex::new(2.0, 0.0);

        let (ast_sub, val_sub) = ceml!((sub z1 z2));
        assert_eq!(
            ast_sub,
            CExpr::Sub(Box::new(CExpr::Var("z1")), Box::new(CExpr::Var("z2")))
        );
        assert_eq!(val_sub, z1 - z2);

        let (ast_div, val_div) = ceml!((div z1 z2));
        assert_eq!(
            ast_div,
            CExpr::Div(Box::new(CExpr::Var("z1")), Box::new(CExpr::Var("z2")))
        );
        assert_eq!(val_div, z1 / z2);
    }

    #[test]
    fn test_ceml_reverse_dsl_stability() {
        // Ensure complex mixed structures correctly round-trip via Reverse DSL
        let x = Complex::new(1.0, 1.0);
        let y = Complex::new(2.0, 2.0);

        // (add (mul x y) (div y x))
        let (ast1, _) = ceml!((add (mul x y) (div y x)));
        assert_eq!(ast1.to_ceml(), "(add (mul x y) (div y x))");

        // (mul (add 1.0 2.0) (sub 3.0 (i)))
        let (ast2, _) = ceml!((mul (add 1.0 2.0) (sub 3.0 (i))));
        assert_eq!(ast2.to_ceml(), "(mul (add 1 2) (sub 3 (i)))");

        // (div (exp (i)) (ln (complex 1.0 1.0)))
        let (ast3, _) = ceml!((div (exp (i)) (ln (complex 1.0 1.0))));
        assert_eq!(ast3.to_ceml(), "(div (exp (i)) (ln (complex 1 1)))");
    }

    #[test]
    fn test_ceml_mixed_var_literal() {
        // Ensure the macro correctly dispatches between $ident and $literal
        let z1 = Complex::new(2.0, 3.0);

        // Literal on the left, Var on the right
        let (ast_add, val_add) = ceml!((add 1.0 z1));
        assert_eq!(ast_add.to_ceml(), "(add 1 z1)");
        assert_eq!(val_add, Complex::new(1.0, 0.0) + z1);

        // Var on the left, Literal on the right
        let (ast_mul, val_mul) = ceml!((mul z1 3.5));
        assert_eq!(ast_mul.to_ceml(), "(mul z1 3.5)");
        assert_eq!(val_mul, z1 * Complex::new(3.5, 0.0));

        // Complex literal on the left, Var on the right
        let (ast_div, val_div) = ceml!((div (complex 1.0 2.0) z1));
        assert_eq!(ast_div.to_ceml(), "(div (complex 1 2) z1)");
        assert_eq!(val_div, Complex::new(1.0, 2.0) / z1);
    }

    #[test]
    fn test_ceml_ast_optimization() {
        let (ast_redundant, _) = ceml!((exp (ln (add 1.0 2.0))));

        assert_eq!(ast_redundant.to_ceml(), "(exp (ln (add 1 2)))");

        let optimized_ast = ast_redundant.optimize();

        assert_eq!(optimized_ast.to_ceml(), "(add 1 2)");

        let (ast2, _) = ceml!((ln (exp (complex 3.0 4.0))));

        assert_eq!(ast2.to_ceml(), "(ln (exp (complex 3 4)))");
        assert_eq!(ast2.optimize().to_ceml(), "(complex 3 4)");
    }
}
