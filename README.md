# Emlex

A mathematical S-expression Domain Specific Language (DSL) compiler embedded in Rust macros. 

`emlex` provides two compile-time evaluation environments:
* `eml!`: A single-form DSL for real number evaluations.
* `ceml!`: A comprehensive DSL for complex number operations.

The library evaluates S-expressions into native Rust operations at compile-time while preserving the Abstract Syntax Tree (AST) for structural optimization and Reverse DSL transformations.

## Architecture & Performance

The macros generate purely native Rust code. To handle AST preservation (`EExpr` / `CExpr`) without runtime allocation overhead, the library utilizes lazy evaluation via static function pointers (`LazyEExpr` / `LazyCExpr`). If the AST is not explicitly consumed at runtime, LLVM's Dead Code Elimination (DCE) removes the allocation instructions, resulting in zero-overhead execution matching handwritten native operations.

AST optimization operates via in-place mutation (`.optimize_in_place()`) to avoid unnecessary reallocations. Reverse DSL reconstruction implements `std::fmt::Display`.

### Recursion Limit

`emlex` performs full S-expression parsing using `macro_rules!`, which relies on
recursive macro expansion. For deeply nested expressions, the default Rust
recursion limit (`#[recursion_limit = "128"]`) may be insufficient.

If you encounter a compilation error such as:

```text
recursion limit reached while expanding the macro
```

add the following attribute to your crate root (`lib.rs` or `main.rs`):

```rust,ignore
#![recursion_limit = "512"]
```

Most projects will not require this, but complex or deeply nested DSL inputs may.

### Benchmarks

Measured on standard release configuration (`cargo bench` via Criterion):

* **Numeric Evaluation (`eval_native_vs_macro`):** `~512 ps` (Pure zero-cost execution)
* **AST In-place Optimization (`ast_optimize_in_place`):** `~382 ns` (~2,600,000 ops/sec)
* **Reverse DSL Stringification (`ast_to_eml_display`):** `~940 ns` (Direct formatting)


## Features

* **Dual Evaluation Engines:**
* `eml!`: Evaluates `(eml x y)` as `exp(x) - ln(y)`.
* `ceml!`: Evaluates standard operations backed by `num_complex::Complex<f64>`.
* **Reverse DSL (AST Reconstruction):** Parses inputs into ASTs that can be reconstructed back into their original S-expression strings using `.to_eml()` / `.to_ceml()` or direct `Display` formatting.
* **AST Optimization:** Reduces redundant mathematical structures (e.g., simplifying `exp(ln(x))` to `x`) through structural pattern matching and zero-copy in-place mutation (`.optimize_in_place()`).

## Usage

Add the dependency to your project:

```bash
cargo add emlex
```

Or add it directly to `Cargo.toml`:

```toml
[dependencies]
emlex = "0.2.0"
```

### Real Number DSL (`eml!`)

```rust
use emlex::prelude::*;

fn main() {
    let x: f64 = 2.0;
    let y: f64 = 3.0;
    
    // Returns (LazyEExpr, f64)
    let (ast, val) = eml!((eml x y));
    
    assert_eq!(val, x.exp() - y.ln());
    assert_eq!(ast.to_eml(), "(eml x y)");
    
    // AST Optimization
    let z: f64 = 5.0;
    let (opt_ast, _) = eml!((eml (eml 1.0 (eml (eml 1.0 z) 1.0)) 1.0));
    assert_eq!(opt_ast.optimize().to_eml(), "z");
}
```

### Complex Number DSL (`ceml!`)

```rust
use emlex::prelude::*;

fn main() {
    let z1 = Complex::new(1.0, 2.0);
    
    // Returns (LazyCExpr, Complex<f64>)
    let (ast, val) = ceml!((add z1 (i)));
    
    assert_eq!(val, z1 + Complex::new(0.0, 1.0));
    assert_eq!(ast.to_ceml(), "(add z1 (i))");
}
```

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
