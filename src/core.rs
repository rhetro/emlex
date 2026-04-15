// ============================================================================
// COMPLEX NUMBER DSL (`ceml!`)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CExpr {
    Var(&'static str),
    Lit(f64),
    ComplexLit(f64, f64),
    I,
    Exp(Box<CExpr>),
    Ln(Box<CExpr>),
    Add(Box<CExpr>, Box<CExpr>),
    Sub(Box<CExpr>, Box<CExpr>),
    Mul(Box<CExpr>, Box<CExpr>),
    Div(Box<CExpr>, Box<CExpr>),
}

impl CExpr {
    pub fn to_ceml(&self) -> String {
        match self {
            CExpr::Var(v) => v.to_string(),
            CExpr::Lit(n) => n.to_string(),
            CExpr::ComplexLit(a, b) => format!("(complex {} {})", a, b),
            CExpr::I => "(i)".to_string(),
            CExpr::Exp(x) => format!("(exp {})", x.to_ceml()),
            CExpr::Ln(x) => format!("(ln {})", x.to_ceml()),
            CExpr::Add(a, b) => format!("(add {} {})", a.to_ceml(), b.to_ceml()),
            CExpr::Sub(a, b) => format!("(sub {} {})", a.to_ceml(), b.to_ceml()),
            CExpr::Mul(a, b) => format!("(mul {} {})", a.to_ceml(), b.to_ceml()),
            CExpr::Div(a, b) => format!("(div {} {})", a.to_ceml(), b.to_ceml()),
        }
    }

    pub fn optimize(&self) -> CExpr {
        match self {
            CExpr::Exp(inner) => {
                let opt = inner.optimize();
                if let CExpr::Ln(x) = opt {
                    *x
                } else {
                    CExpr::Exp(Box::new(opt))
                }
            }
            CExpr::Ln(inner) => {
                let opt = inner.optimize();
                if let CExpr::Exp(x) = opt {
                    *x
                } else {
                    CExpr::Ln(Box::new(opt))
                }
            }
            CExpr::Add(a, b) => CExpr::Add(Box::new(a.optimize()), Box::new(b.optimize())),
            CExpr::Sub(a, b) => CExpr::Sub(Box::new(a.optimize()), Box::new(b.optimize())),
            CExpr::Mul(a, b) => CExpr::Mul(Box::new(a.optimize()), Box::new(b.optimize())),
            CExpr::Div(a, b) => CExpr::Div(Box::new(a.optimize()), Box::new(b.optimize())),
            _ => self.clone(),
        }
    }
}

// Lazy wrapper to achieve absolute zero-cost execution at runtime.
// The heap allocation is entirely deferred until explicit evaluation.
pub struct LazyCExpr(pub fn() -> CExpr);

impl LazyCExpr {
    pub fn to_ceml(&self) -> String {
        (self.0)().to_ceml()
    }
    pub fn optimize(&self) -> CExpr {
        (self.0)().optimize()
    }
}

impl std::fmt::Debug for LazyCExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&(self.0)(), f)
    }
}

impl PartialEq<CExpr> for LazyCExpr {
    fn eq(&self, other: &CExpr) -> bool {
        &(self.0)() == other
    }
}

impl PartialEq<LazyCExpr> for CExpr {
    fn eq(&self, other: &LazyCExpr) -> bool {
        self == &(other.0)()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __ceml_call {
    // Generate a pure, non-capturing function pointer to defer AST creation.
    ([finalize], [$ast:expr, $val:expr]) => {
        {
            fn __generate_ast() -> $crate::CExpr { $ast }
            ($crate::LazyCExpr(__generate_ast), $val)
        }
    };
    ([cb_exp $($cb:tt)*], [$ast:expr, $val:expr]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Exp(Box::new($ast))), ($val.exp()) ])
    };
    ([cb_ln $($cb:tt)*], [$ast:expr, $val:expr]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Ln(Box::new($ast))), ($val.ln()) ])
    };
    ([cb_bin_x $y:tt $op:ident $($cb:tt)*], [$ast_x:expr, $val_x:expr]) => {
        $crate::__ceml_walk!([$y], [cb_bin_y $ast_x, $val_x, $op, $($cb)*])
    };
    ([cb_bin_y $ast_x:expr, $val_x:expr, cb_add, $($cb:tt)*], [$ast_y:expr, $val_y:expr]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Add(Box::new($ast_x), Box::new($ast_y))), ($val_x + $val_y) ])
    };
    ([cb_bin_y $ast_x:expr, $val_x:expr, cb_sub, $($cb:tt)*], [$ast_y:expr, $val_y:expr]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Sub(Box::new($ast_x), Box::new($ast_y))), ($val_x - $val_y) ])
    };
    ([cb_bin_y $ast_x:expr, $val_x:expr, cb_mul, $($cb:tt)*], [$ast_y:expr, $val_y:expr]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Mul(Box::new($ast_x), Box::new($ast_y))), ($val_x * $val_y) ])
    };
    ([cb_bin_y $ast_x:expr, $val_x:expr, cb_div, $($cb:tt)*], [$ast_y:expr, $val_y:expr]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Div(Box::new($ast_x), Box::new($ast_y))), ($val_x / $val_y) ])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __ceml_walk {
    ([(exp $x:tt)], [$($cb:tt)*]) => { $crate::__ceml_walk!([$x], [cb_exp $($cb)*]) };
    ([(ln $x:tt)], [$($cb:tt)*]) => { $crate::__ceml_walk!([$x], [cb_ln $($cb)*]) };
    ([(add $x:tt $y:tt)], [$($cb:tt)*]) => { $crate::__ceml_walk!([$x], [cb_bin_x $y cb_add $($cb)*]) };
    ([(sub $x:tt $y:tt)], [$($cb:tt)*]) => { $crate::__ceml_walk!([$x], [cb_bin_x $y cb_sub $($cb)*]) };
    ([(mul $x:tt $y:tt)], [$($cb:tt)*]) => { $crate::__ceml_walk!([$x], [cb_bin_x $y cb_mul $($cb)*]) };
    ([(div $x:tt $y:tt)], [$($cb:tt)*]) => { $crate::__ceml_walk!([$x], [cb_bin_x $y cb_div $($cb)*]) };
    ([(complex $re:tt $im:tt)], [$($cb:tt)*]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::ComplexLit($re as f64, $im as f64)), (num_complex::Complex::new($re as f64, $im as f64)) ])
    };
    ([(i)], [$($cb:tt)*]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::I), (num_complex::Complex::new(0.0, 1.0)) ])
    };
    ([$v:ident], [$($cb:tt)*]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Var(stringify!($v))), ($v) ])
    };
    ([$v:literal], [$($cb:tt)*]) => {
        $crate::__ceml_call!([$($cb)*], [ ($crate::CExpr::Lit($v as f64)), (num_complex::Complex::new($v as f64, 0.0)) ])
    };
}

#[macro_export]
macro_rules! ceml {
    ($t:tt) => {
        $crate::__ceml_walk!([$t], [finalize])
    };
}

// ============================================================================
// REAL NUMBER DSL (`eml!`)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum EExpr {
    Var(&'static str),
    Lit(f64),
    Eml(Box<EExpr>, Box<EExpr>),
}

impl EExpr {
    fn is_one(&self) -> bool {
        if let EExpr::Lit(n) = self {
            (*n - 1.0).abs() < f64::EPSILON
        } else {
            false
        }
    }

    pub fn to_eml(&self) -> String {
        match self {
            EExpr::Var(v) => v.to_string(),
            EExpr::Lit(n) => n.to_string(),
            EExpr::Eml(x, y) => format!("(eml {} {})", x.to_eml(), y.to_eml()),
        }
    }

    pub fn optimize(&self) -> EExpr {
        match self {
            EExpr::Eml(x, y) => {
                let opt_x = x.optimize();
                let opt_y = y.optimize();

                if opt_y.is_one() {
                    if let EExpr::Eml(l1, r1) = &opt_x {
                        if l1.is_one() {
                            if let EExpr::Eml(l2, r2) = r1.as_ref() {
                                if r2.is_one() {
                                    if let EExpr::Eml(l3, r3) = l2.as_ref() {
                                        if l3.is_one() {
                                            return *r3.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if opt_x.is_one() {
                    if let EExpr::Eml(l1, r1) = &opt_y {
                        if r1.is_one() {
                            if let EExpr::Eml(l2, r2) = l1.as_ref() {
                                if l2.is_one() {
                                    if let EExpr::Eml(z_node, z_one) = r2.as_ref() {
                                        if z_one.is_one() {
                                            return *z_node.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                EExpr::Eml(Box::new(opt_x), Box::new(opt_y))
            }
            _ => self.clone(),
        }
    }
}

// Lazy wrapper to achieve absolute zero-cost execution at runtime.
pub struct LazyEExpr(pub fn() -> EExpr);

impl LazyEExpr {
    pub fn to_eml(&self) -> String {
        (self.0)().to_eml()
    }
    pub fn optimize(&self) -> EExpr {
        (self.0)().optimize()
    }
}

impl std::fmt::Debug for LazyEExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&(self.0)(), f)
    }
}

impl PartialEq<EExpr> for LazyEExpr {
    fn eq(&self, other: &EExpr) -> bool {
        &(self.0)() == other
    }
}

impl PartialEq<LazyEExpr> for EExpr {
    fn eq(&self, other: &LazyEExpr) -> bool {
        self == &(other.0)()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __eml_call {
    // Generate a pure, non-capturing function pointer to defer AST creation.
    ([finalize], [$ast:expr, $val:expr]) => {
        {
            fn __generate_ast() -> $crate::EExpr { $ast }
            ($crate::LazyEExpr(__generate_ast), $val)
        }
    };

    ([cb_eml_x $y:tt $($cb:tt)*], [$ast_x:expr, $val_x:expr]) => {
        $crate::__eml_walk!([$y], [cb_eml_y $ast_x, $val_x, $($cb)*])
    };

    ([cb_eml_y $ast_x:expr, $val_x:expr, $($cb:tt)*], [$ast_y:expr, $val_y:expr]) => {
        $crate::__eml_call!([$($cb)*], [
            ($crate::EExpr::Eml(Box::new($ast_x), Box::new($ast_y))),
            ($val_x.exp() - $val_y.ln())
        ])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __eml_walk {
    ([(eml $x:tt $y:tt)], [$($cb:tt)*]) => { $crate::__eml_walk!([$x], [cb_eml_x $y $($cb)*]) };

    ([$v:ident], [$($cb:tt)*]) => {
        $crate::__eml_call!([$($cb)*], [ ($crate::EExpr::Var(stringify!($v))), ($v) ])
    };

    ([$v:literal], [$($cb:tt)*]) => {
        $crate::__eml_call!([$($cb)*], [ ($crate::EExpr::Lit($v as f64)), ($v as f64) ])
    };
}

#[macro_export]
macro_rules! eml {
    ($t:tt) => {
        $crate::__eml_walk!([$t], [finalize])
    };
}
