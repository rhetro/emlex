// UI tests for compile‑fail cases.
// These tests are *supposed* to fail.
//
// When updating expected error output, overwrite the .stderr files with:
//
//     TRYBUILD=overwrite cargo test --test ui_test
//
// After the expected outputs are recorded, run normally:
//
//     cargo test --test ui_test
//
// The test passes only if the compiler errors match the saved .stderr files.
//

#[test]
fn test_compile_failures() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui/*.rs");
}
