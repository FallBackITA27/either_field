struct GenericStruct<T> {
    either_type: T,
}

impl<T> GenericStruct<T> {
    fn get_either_type_field(self) -> T {
        return self.either_type;
    }
}

type OutputStruct1 = GenericStruct<i32>;
type OutputStruct2 = GenericStruct<&'static str>;

fn main() {
    let out1 = OutputStruct1 { either_type: 0 };
    let out2 = OutputStruct2 {
        either_type: "this is a test",
    };
}
