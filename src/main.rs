mod java;

#[macro_use]
extern crate nom;

use java::class_file::{read_class_file, ClassFile};


fn main() {
    let report: ClassFile = read_class_file(include_bytes!("../sample/HelloWorld.class")).unwrap().1;
    println!("{:#?}", report);

    println!("{:?}", report.get_class_name());
}
