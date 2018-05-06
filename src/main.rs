#![feature(test)]
mod java;

#[macro_use]
extern crate nom;
extern crate test;

use java::class_file::{read_class_file, ClassFile};
use std::fs::File;
use std::env;
use std::io::Read;


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
}
