
#[test]
fn not_included() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/should_fail/not_included.rs");
}

#[test]
fn it_works() {
    let t = trybuild::TestCases::new();
    t.pass("src/example_usage.rs");
    t.pass("src/typical_usage.rs");
}
