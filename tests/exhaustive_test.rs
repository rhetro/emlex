use emlex::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhaustive_branching() {
        let v1: f64 = 1.0;
        let v2: f64 = 2.0;
        let v3: f64 = 3.0;

        // Path 1: Deep nesting purely on the left side (x)
        // eml(eml(v1, v2), v3) -> (v1.exp() - v2.ln()).exp() - v3.ln()
        let (_, res_left) = eml!((eml (eml v1 v2) v3));
        assert_eq!(res_left, (v1.exp() - v2.ln()).exp() - v3.ln());

        // Path 2: Deep nesting purely on the right side (y)
        // eml(v1, eml(v2, v3)) -> v1.exp() - (v2.exp() - v3.ln()).ln()
        let (_, res_right) = eml!((eml v1 (eml v2 v3)));
        assert_eq!(res_right, v1.exp() - (v2.exp() - v3.ln()).ln());

        // Path 3: Symmetric evaluation (both sides are expressions)
        // eml(eml(v1, 1), eml(1, eml(eml(1, v2), 1)))
        let (_, res_sym) = eml!((eml (eml v1 1) (eml 1 (eml (eml 1 v2) 1))));
        assert_eq!(res_sym, v1.exp().exp() - v2.ln().ln());
    }
}
