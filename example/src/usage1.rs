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
#[derive(Debug)]
struct GenericStructWithOmittedFields {
    either_type_0: either_field::either!(() | i32),
    either_type_1: either_field::either!(() | i32 | u32),
    either_type_2: either_field::either!(() | i32 | String),
}

pub fn test() {
    let omitted0 = OmitsEverythingBut0 {
        either_type_0: 32i32,
        either_type_1: (),
        either_type_2: (),
    };

    let omitted1 = OmitsEverythingBut1 {
        either_type_0: (),
        either_type_1: 32u32,
        either_type_2: (),
    };

    println!("{omitted0:#?}");
    println!("{omitted1:#?}");
}
