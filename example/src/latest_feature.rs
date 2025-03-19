#[either_field::make_template(
    GenStructs: true,
    DeleteTemplate: true,
    OmitEmptyTupleFields: true;
    OmitsEverythingBut0: [
        0: i32
    ],
    OmitsEverythingBut1: [
        1: u32
    ],
    OmitsEverythingBut2: [_, _, &'static str]
)]
#[derive(Debug)]
struct GenericStructWithOmittedFields(
    either_field::either!(() | i32),
    either_field::either!(() | i32 | u32),
    either_field::either!(() | i32 | &'static str),
);

#[either_field::make_template(
    GenStructs: true,
    OmitEmptyTupleFields: true;
    AnotherOmitsEverythingBut0:
    [
        either_type_0: i32
    ],
    AnotherOmitsEverythingBut1:
    [
        either_type_1: u32
    ]
)]
#[derive(Debug)]
struct AnotherGenericStruct {
    either_type_0: either_field::either!(() | i32),
    either_type_1: either_field::either!(() | i32 | u32),
    either_type_2: either_field::either!(() | i32 | String),
}

pub fn test() {
    let omitted_0 = OmitsEverythingBut0(32i32);
    let omitted_1 = OmitsEverythingBut1(32u32);
    let omitted_2 = OmitsEverythingBut2("Test");
    let another_omitted_0 = AnotherOmitsEverythingBut0 {
        either_type_0: 33i32,
    };
    let another_omitted_1 = AnotherOmitsEverythingBut1 {
        either_type_1: 33u32,
    };
    let another_generic = AnotherGenericStruct {
        either_type_0: (),
        either_type_1: (),
        either_type_2: (),
    };
    println!("{omitted_0:#?}");
    println!("{omitted_1:#?}");
    println!("{omitted_2:#?}");
    println!("{another_omitted_0:#?}");
    println!("{another_omitted_1:#?}");
    println!("{another_generic:#?}");
}
