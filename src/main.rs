mod java;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate failure;

use java::class_file::{read_class_file, ClassFile};
use std::fs::File;
use std::env;
use std::io::Read;
use java::class_file::Attribute;
use java::class_file::CodeBlock;


fn main() {
    let args = env::args().collect::<Vec<String>>();
    let filename = args.get(1);
    let mut buffer = Vec::new();
    let content = if let Some(path) = filename {
        let mut f = File::open(path).expect("cannot open file");
        f.read_to_end(&mut buffer).expect("cannot read file");
        buffer.as_slice()
    } else {
        include_bytes!("../sample/DemoClass.class")
    };


    let report: ClassFile = read_class_file(content).unwrap().1;
    println!("{:#?}", report);

    println!("{:?}", report.get_class_name());

    report.methods.iter().for_each(|method| {
        println!("{:?}", report.constants.get(usize::from(method.name_index - 1)));
        println!("{}", java::class_file::dissasm::dissassemble(method))
    })
}
