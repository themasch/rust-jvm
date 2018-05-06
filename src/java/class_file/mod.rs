mod parser;
pub use self::parser::read_class_file;

#[derive(Debug)]
pub struct ClassFile {
    pub version: (u16, u16),
    pub constants: Vec<ConstantType>,
    pub access_flags: u16,
    pub this_index: u16,
    pub super_index: u16,
    interfaces: Vec<u16>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    attributes: Vec<Attribute>
}

impl ClassFile {
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
}

#[derive(Debug)]
struct Field {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
struct Method {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
struct Attribute {
    name_index: u16,
    info: Vec<u8>,
}

#[derive(Debug)]
pub enum ConstantType {
    Utf8 { value: String },
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
