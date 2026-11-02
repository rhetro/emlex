use std::fmt;

// ============================================================================
// PATTERN DIAGNOSTIC PRISMS (Xopsy CPS + Opejson In-Place Surgery)
// ============================================================================

/// Continuation-Passing Style (CPS) macro for S-expressions based on Xopsy's focus! architecture.
/// Recursively matches and extracts mutable references to deep target nodes without manual if-let nesting.
#[macro_export]
#[doc(hidden)]
macro_rules! __focus_eml_pat {
    ($node:expr, 1.0, $ok:tt, $fail:tt) => {
        if $node.is_one() {
            $ok
        } else {
            $fail
        }
    };

    ($node:expr, (? $var:ident), $ok:tt, $fail:tt) => {{
        let $var = $node;
        $ok
    }};

    ($node:expr, (eml $left:tt $right:tt), $ok:tt, $fail:tt) => {
        match $node {
            $crate::EExpr::Eml(ref mut __l, ref mut __r) => {
                $crate::__focus_eml_pat!(
                    __l.as_mut(),
                    $left,
                    { $crate::__focus_eml_pat!(__r.as_mut(), $right, $ok, $fail) },
                    $fail
                )
            }
            _ => $fail,
        }
    };
}

#[macro_export]
macro_rules! focus_eml {
    ($target:expr, $pat:tt => $ok:expr, _ => $fail:expr) => {
        $crate::__focus_eml_pat!($target, $pat, { $ok }, { $fail })
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __focus_ceml_pat {
    ($node:expr, (? $var:ident), $ok:tt, $fail:tt) => {{
        let $var = $node;
        $ok
    }};

    ($node:expr, (exp $inner:tt), $ok:tt, $fail:tt) => {
        match $node {
            $crate::CExpr::Exp(ref mut __inner) => {
                $crate::__focus_ceml_pat!(__inner.as_mut(), $inner, $ok, $fail)
            }
            _ => $fail,
        }
    };

    ($node:expr, (ln $inner:tt), $ok:tt, $fail:tt) => {
        match $node {
            $crate::CExpr::Ln(ref mut __inner) => {
                $crate::__focus_ceml_pat!(__inner.as_mut(), $inner, $ok, $fail)
            }
            _ => $fail,
        }
    };
}

#[macro_export]
macro_rules! focus_ceml {
    ($target:expr, $pat:tt => $ok:expr, _ => $fail:expr) => {
        $crate::__focus_ceml_pat!($target, $pat, { $ok }, { $fail })
    };
}

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

impl fmt::Display for CExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CExpr::Var(v) => write!(f, "{v}"),
            CExpr::Lit(n) => write!(f, "{n}"),
            CExpr::ComplexLit(a, b) => write!(f, "(complex {a} {b})"),
            CExpr::I => write!(f, "(i)"),
            CExpr::Exp(x) => write!(f, "(exp {x})"),
            CExpr::Ln(x) => write!(f, "(ln {x})"),
            CExpr::Add(a, b) => write!(f, "(add {a} {b})"),
            CExpr::Sub(a, b) => write!(f, "(sub {a} {b})"),
            CExpr::Mul(a, b) => write!(f, "(mul {a} {b})"),
            CExpr::Div(a, b) => write!(f, "(div {a} {b})"),
        }
    }
}

impl CExpr {
    #[inline]
    pub fn to_ceml(&self) -> String {
        self.to_string()
    }

    /// In-place AST mutation eliminating heap reallocations and cloning.
    pub fn optimize_in_place(&mut self) {
        match self {
            CExpr::Exp(inner) => {
                inner.optimize_in_place();
            }
            CExpr::Ln(inner) => {
                inner.optimize_in_place();
            }
            CExpr::Add(a, b) | CExpr::Sub(a, b) | CExpr::Mul(a, b) | CExpr::Div(a, b) => {
                a.optimize_in_place();
                b.optimize_in_place();
                return;
            }
            _ => return,
        }

        let mut extracted = None;
        focus_ceml!(
            self,
            (exp (ln (? target))) => {
                extracted = Some(std::mem::replace(target, CExpr::I));
            },
            _ => {
                focus_ceml!(
                    self,
                    (ln (exp (? target))) => {
                        extracted = Some(std::mem::replace(target, CExpr::I));
                    },
                    _ => ()
                );
            }
        );

        if let Some(node) = extracted {
            *self = node;
        }
    }

    pub fn optimize(&self) -> CExpr {
        let mut cloned = self.clone();
        cloned.optimize_in_place();
        cloned
    }
}

// Lazy wrapper to achieve absolute zero-cost execution at runtime.
pub struct LazyCExpr(pub fn() -> CExpr);

impl LazyCExpr {
    pub fn to_ceml(&self) -> String {
        (self.0)().to_ceml()
    }
    pub fn optimize(&self) -> CExpr {
        (self.0)().optimize()
    }
}

impl fmt::Display for LazyCExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (self.0)())
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
    ([finalize], [$ast:expr, $val:expr]) => {{
        fn __generate_ast() -> $crate::CExpr {
            $ast
        }
        ($crate::LazyCExpr(__generate_ast), $val)
    }};
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

impl fmt::Display for EExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EExpr::Var(v) => write!(f, "{v}"),
            EExpr::Lit(n) => write!(f, "{n}"),
            EExpr::Eml(x, y) => write!(f, "(eml {x} {y})"),
        }
    }
}

impl EExpr {
    #[inline]
    pub fn is_one(&self) -> bool {
        if let EExpr::Lit(n) = self {
            (*n - 1.0).abs() < f64::EPSILON
        } else {
            false
        }
    }

    #[inline]
    pub fn to_eml(&self) -> String {
        self.to_string()
    }

    /// In-place AST mutation eliminating heap reallocations and cloning.
    pub fn optimize_in_place(&mut self) {
        if let EExpr::Eml(ref mut x, ref mut y) = self {
            x.optimize_in_place();
            y.optimize_in_place();
        } else {
            return;
        }

        let mut extracted = None;
        focus_eml!(
            self,
            (eml (eml 1.0 (eml (eml 1.0 (? target)) 1.0)) 1.0) => {
                extracted = Some(std::mem::replace(target, EExpr::Lit(0.0)));
            },
            _ => {
                focus_eml!(
                    self,
                    (eml 1.0 (eml (eml 1.0 (eml (? target) 1.0)) 1.0)) => {
                        extracted = Some(std::mem::replace(target, EExpr::Lit(0.0)));
                    },
                    _ => ()
                );
            }
        );

        if let Some(node) = extracted {
            *self = node;
        }
    }

    pub fn optimize(&self) -> EExpr {
        let mut cloned = self.clone();
        cloned.optimize_in_place();
        cloned
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

impl fmt::Display for LazyEExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (self.0)())
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
    ([finalize], [$ast:expr, $val:expr]) => {{
        fn __generate_ast() -> $crate::EExpr {
            $ast
        }
        ($crate::LazyEExpr(__generate_ast), $val)
    }};

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
    ([(eml $x:tt $y:tt)], [$($cb:tt)*]) => {
        $crate::__eml_walk!([$x], [cb_eml_x $y $($cb)*])
    };

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
