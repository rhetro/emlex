use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emlex::prelude::*;

fn bench_evaluation(c: &mut Criterion) {
    // 1. Zero-cost native mathematical evaluation
    c.bench_function("eval_native_vs_macro", |b| {
        let x: f64 = black_box(2.0);
        let y: f64 = black_box(3.0);
        b.iter(|| {
            let (_, val) = eml!((eml (eml x y) (eml y x)));
            black_box(val);
        });
    });

    // 2. In-place AST optimization
    let (lazy_ast, _) = eml!((eml (eml 1.0 (eml (eml 1.0 5.0) 1.0)) 1.0));
    let base_tree = (lazy_ast.0)();

    c.bench_function("ast_optimize_in_place", |b| {
        b.iter_batched(
            || base_tree.clone(),
            |mut tree| {
                tree.optimize_in_place();
                black_box(tree);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // 3. Reverse DSL stringification via Display
    c.bench_function("ast_to_eml_display", |b| {
        b.iter(|| {
            let s = base_tree.to_eml();
            black_box(s);
        });
    });
}

criterion_group!(benches, bench_evaluation);
criterion_main!(benches);
