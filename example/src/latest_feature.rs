#[either_field::make_template(
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

pub fn test() {
    let omitted0 = OmitsEverythingBut0(32i32,(),(),);
    let omitted1 = OmitsEverythingBut1((),32u32,(),);
    let omitted2 = OmitsEverythingBut2((),(),"Test");
    println!("{omitted0:#?}");
    println!("{omitted1:#?}");
    println!("{omitted2:#?}");
}
