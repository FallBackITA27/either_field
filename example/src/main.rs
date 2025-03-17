// Showcases how you can make a functionality similar to Omit<T, field> in Typescript
// 1.0.0
mod usage1;

// Showcases how you can generally use the crate
// 1.0.0
mod usage2;

mod latest_feature;

fn main() {
    usage1::test();
    usage2::test();
    latest_feature::test();
}
