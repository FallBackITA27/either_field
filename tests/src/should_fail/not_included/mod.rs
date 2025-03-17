
#[either_field::make_template(
    NotIncluded:
    [
        either_type_4: &'static str
    ],
)]
struct GenericStruct<A: IntoIterator, B: std::fmt::Debug + 'static> {
    generic_a: A,
    generic_b: B,
    either_type_0: either_field::either!(i32 | &'static str),
    either_type_1: either_field::either!(i32 | &'static str | u32),
    either_type_2: either_field::either!(i32 | &'static str | String),
    either_type_3: either_field::either!(i32 | A | String),
    either_type_4: either_field::either!(i32 | (A, B) | String),
}


fn main() {}
