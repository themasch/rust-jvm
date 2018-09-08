mod parser;
pub mod dissasm;

use java::instructions::*;
pub use self::parser::read_class_file;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug)]
pub struct ClassFile<'a> {
    pub version: (u16, u16),
    pub constants: Vec<ConstantType<'a>>,
    pub access_flags: u16,
    pub this_index: u16,
    pub super_index: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field<'a>>,
    pub methods: Vec<Method<'a>>,
    pub attributes: Vec<Attribute<'a>>,
}

impl<'a> ClassFile<'a> {
    pub fn get_constant(&self, index: u16) -> Option<&ConstantType> {
        self.constants.get(index as usize - 1)
    }

    pub fn get_class_name(&self) -> &str {
        let cls = self.get_constant(self.this_index).unwrap();
        let cls_name = match cls {
            ConstantType::Class { name_index: idx } => self.get_constant(*idx).unwrap(),
            _ => panic!("cannot read class name")
        };


        match cls_name {
            ConstantType::Utf8 { value } => &value,
            _ => panic!("cannot read class name")
        }
    }

    pub fn get_method_from_nat(&self, nat_index: u16) -> Option<&Method> {
        let name_and_type = self.get_constant(nat_index);

        let (name_index, type_index) = match name_and_type {
            Some(ConstantType::NameAndType { name_index, descriptor_index }) => {
                (name_index, descriptor_index)
            }
            _ => return None
        };

        let name = match self.get_constant(*name_index) {
            Some(ConstantType::Utf8 { value }) => {
                *value
            }
            _ => return None
        };

        let type_desc = match self.get_constant(*type_index) {
            Some(ConstantType::Utf8 { value }) => {
                *value
            }
            _ => return None
        };

        self.methods.iter().find(|method| method.name == name && method.descriptor == type_desc)
    }
}

#[derive(Debug)]
pub struct Field<'a> {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<Attribute<'a>>,
}

#[derive(Debug)]
pub struct Method<'a> {
    pub access_flags: u16,
    pub name: &'a str,
    pub descriptor: &'a str,
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum MethodAccess {
    Public,
    Private,
    Protected,
    Static,
    Final,
    Synchronized,
    Native,
    Abstract,
    Strict,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ValueType {
    Void,
    Integer,
    Object(String),
    Array(Box<ValueType>),
}

#[derive(Debug)]
pub struct MethodDescriptor {
    pub return_type: ValueType,
    pub arguments: Vec<ValueType>,
}

use nom::IResult;
use std::slice::Iter;
use std::collections::HashMap;

impl FromStr for MethodDescriptor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match parser::method_desc(s.as_bytes()) {
            Ok((rem, (args, ret))) => Ok(MethodDescriptor { arguments: args, return_type: ret }),
            Err(_) => panic!("asdf"),
        }
    }
}

pub struct Instructions {
    instructions: Vec<Instruction>,
    index: HashMap<usize, usize>,
    current: usize,
}

impl Instructions {
    pub fn create(instructions: Vec<Instruction>) -> Instructions {
        Instructions {
            instructions,
            index: HashMap::new(),
            current: 0,
        }
    }

    fn offset_to_index(&mut self, offset: usize) -> Option<usize> {
        if self.index.contains_key(&offset) {
            return self.index.get(&offset).map(|value| *value);
        }

        let mut curr_offset = 0;
        let mut curr_index = 0;
        for instruction in self.instructions.iter() {
            let size = instruction.get_size();
            if !self.index.contains_key(&curr_offset) {
                self.index.insert(curr_offset, curr_index);
            }

            if curr_offset + size > offset {
                return Some(curr_index);
            }

            curr_offset = curr_offset + size;
            curr_index += 1;
        }

        None
    }

    pub fn goto(&mut self, offset: usize) -> bool {
        let index = self.offset_to_index(offset);
        if let Some(index) = index {
            self.current = index;
            true
        } else {
            false
        }
    }
}

impl Iterator for Instructions {
    type Item = Instruction;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let item = self.instructions.get(self.current);

        self.current += 1;


        item.map(|x| x.clone())
    }
}

impl<'a> Method<'a> {
    pub fn instructions(&self) -> Instructions {
        Instructions::create(self.get_code().unwrap().instructions().unwrap())
    }

    pub fn get_code(&self) -> Option<&CodeBlock<'a>> {
        self.attributes.iter()
            .filter_map(
                |attr| match attr {
                    Attribute::CodeAttribute(code) => Some(code),
                    _ => None
                }
            )
            .collect::<Vec<&CodeBlock>>()
            .first()
            .map(|x| *x)
    }

    pub fn get_signature(&self) -> MethodDescriptor {
        match MethodDescriptor::from_str(self.descriptor) {
            Ok(method) => method,
            Err(err) => panic!("{:?}", err)
        }
    }

    pub fn get_access(&self) -> HashSet<MethodAccess> {
        let mut set = HashSet::new();
        if self.access_flags & 0x0001 == 0x0001 {
            set.insert(MethodAccess::Public);
        }
        if self.access_flags & 0x0002 == 0x0002 {
            set.insert(MethodAccess::Private);
        }
        if self.access_flags & 0x0004 == 0x0004 {
            set.insert(MethodAccess::Protected);
        }
        if self.access_flags & 0x0008 == 0x0008 {
            set.insert(MethodAccess::Static);
        }
        if self.access_flags & 0x0010 == 0x0010 {
            set.insert(MethodAccess::Final);
        }
        if self.access_flags & 0x0020 == 0x0020 {
            set.insert(MethodAccess::Synchronized);
        }
        if self.access_flags & 0x0040 == 0x0040 {
            set.insert(MethodAccess::Native);
        }
        if self.access_flags & 0x0080 == 0x0080 {
            set.insert(MethodAccess::Strict);
        }

        return set;
    }
}

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub max_stack: u16,
    pub max_locals: u16,
    code: Vec<u8>,
    attributes: Vec<Attribute<'a>>,
}

impl<'a> CodeBlock<'a> {
    pub fn instructions(&self) -> Result<Vec<Instruction>, ReadInstructionError<&[u8]>> {
        Instruction::read_all(&self.code[..])
    }

    ///  Vec<usize>  pc -> ln
    pub fn get_line_numbers(&self) -> Vec<usize> {
        let line_number_attr = self.attributes.iter().find(|x| match x {
            Attribute::LineNumberTable(_) => true,
            _ => false
        });

        let line_number = match line_number_attr {
            Some(Attribute::LineNumberTable(t)) => t,
            _ => return Vec::new()
        };

        let max = line_number.last().unwrap();

        let mut numbers = Vec::new();
        for x in 0..(max.0 + 1) {
            let ln = line_number
                .iter()
                .filter_map(|&(pc, ln)|
                    if pc <= x {
                        Some(ln)
                    } else {
                        None
                    }
                )
                .last()
                .expect("no matching line number");

            numbers.push(ln as usize);
        }

        numbers
    }
}

#[derive(Debug)]
pub enum Attribute<'a> {
    LineNumberTable(Vec<(u16, u16)>),
    CodeAttribute(CodeBlock<'a>),
    GenericAttribute {
        name: String,
        info: &'a [u8],
    },
}

#[derive(Debug)]
pub enum ConstantType<'a> {
    Utf8 { value: &'a str },
    Integer { value: i32 },
    Float { value: f32 },
    Long { value: i64 },
    Double { value: f64 },
    Class { name_index: u16 },
    String { string_index: u16 },
    FieldRef { class_index: u16, name_and_type_index: u16 },
    MethodRef { class_index: u16, name_and_type_index: u16 },
    InterfaceMethodRef { class_index: u16, name_and_type_index: u16 },
    NameAndType { name_index: u16, descriptor_index: u16 },
    MethodHandle { reference_kind: u8, reference_index: u16 },
    MethodType { descriptor_index: u16 },
    InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    Module { name_index: u16 },
    Package { name_index: u16 },
}
