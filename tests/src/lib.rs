
#[test]
fn errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/should_fail/not_included/mod.rs");
}

#[test]
fn successes() {
    let t = trybuild::TestCases::new();
    t.pass("../example/src/usage1.rs");
    t.pass("../example/src/usage2.rs");
    t.pass("../example/src/latest_feature.rs");
}
