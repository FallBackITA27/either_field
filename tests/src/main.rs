#[either_field::make_template(
    DerivedStruct:
    [
        either_type_0: &'static str
    ],
    TestToo:
    [
        either_type_1: u32
    ]
)]
struct GenericStruct<A: IntoIterator, B: std::fmt::Debug + 'static> {
    generic_a: A,
    generic_b: B,
    either_type_0: either_field::either!(i32 | &'static str),
    either_type_1: either_field::either!(i32 | &'static str | u32),
    either_type_2: either_field::either!(i32 | &'static str | String),
}

#[either_field::make_template(
    OmitsEverythingBut0:
    [
        either_type_0: i32
    ],
    OmitsEverythingBut1:
    [
        either_type_1: u32
    ]
)]
struct GenericStructWithOmittedFields {
    either_type_0: either_field::either!(() | i32),
    either_type_1: either_field::either!(() | i32 | u32),
    either_type_2: either_field::either!(() | i32 | String),
}

fn main() {}
