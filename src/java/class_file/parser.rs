use nom::*;

use super::*;

named!(
    const_class<ConstantType>,
    do_parse!(name_index: be_u16 >> ( ConstantType::Class { name_index } ) )
);
named!(
    const_fieldref<ConstantType>,
    do_parse!(class_index: be_u16 >> name_and_type_index: be_u16 >> ( ConstantType::FieldRef { class_index, name_and_type_index } ) )
);
named!(
    const_methodref<ConstantType>,
    do_parse!(class_index: be_u16 >> name_and_type_index: be_u16 >> ( ConstantType::MethodRef { class_index, name_and_type_index } ) )
);
named!(
    const_interface_methodref<ConstantType>,
    do_parse!(class_index: be_u16 >> name_and_type_index: be_u16 >> ( ConstantType::InterfaceMethodRef { class_index, name_and_type_index } ) )
);
named!(
    const_string<ConstantType>,
    do_parse!(string_index: be_u16 >> ( ConstantType::String { string_index } ))
);
named!(
    const_integer<ConstantType>,
    do_parse!(value: be_i32 >> ( ConstantType::Integer { value } ))
);
named!(
    const_float<ConstantType>,
    do_parse!(value: float >> ( ConstantType::Float { value } ))
);
named!(
    const_long<ConstantType>,
    do_parse!(value: be_i64 >> ( ConstantType::Long { value } ))
);
named!(
    const_double<ConstantType>,
    do_parse!(value: double >> ( ConstantType::Double { value } ))
);
named!(
    const_name_and_type<ConstantType>,
    do_parse!(name_index: be_u16 >> descriptor_index: be_u16 >> ( ConstantType::NameAndType { name_index, descriptor_index } ))
);
named!(
    const_utf8<ConstantType>,
    do_parse!(bytes: length_data!(be_u16) >> ( ConstantType::Utf8 { value: String::from_utf8(bytes.to_vec()).unwrap() } ) )
);
named!(
    const_method_handle<ConstantType>,
    do_parse!(reference_kind: be_u8 >> reference_index: be_u16 >> ( ConstantType::MethodHandle { reference_kind, reference_index } )  )
);
named!(
    const_method_type<ConstantType>,
    do_parse!(descriptor_index: be_u16 >> ( ConstantType::MethodType { descriptor_index } ) )
);
named!(
    const_invoke_dynamic<ConstantType>,
    do_parse!(bootstrap_method_attr_index: be_u16 >> name_and_type_index: be_u16 >> ( ConstantType::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index  } )  )
);
named!(
    const_module<ConstantType>,
    do_parse!(name_index: be_u16 >> ( ConstantType::Module { name_index } ) )
);
named!(
    const_package<ConstantType>,
    do_parse!(name_index: be_u16 >> ( ConstantType::Package { name_index} ) )
);

named!(
    constant<&[u8], ConstantType>,
    dbg_dmp!(switch!(be_u8,
        1 => dbg_dmp!(call!(const_utf8 )) |
        3 => dbg_dmp!(call!(const_integer )) |
        4 => dbg_dmp!(call!(const_float )) |
        5 => dbg_dmp!(call!(const_long )) |
        6 => dbg_dmp!(call!(const_double )) |
        7 => dbg_dmp!(call!(const_class )) |
        8 => dbg_dmp!(call!(const_string )) |
        9 => dbg_dmp!(call!(const_fieldref )) |
        10 => dbg_dmp!(call!(const_methodref )) |
        11 => dbg_dmp!(call!(const_interface_methodref )) |
        12 => dbg_dmp!(call!(const_name_and_type )) |
        15 => dbg_dmp!(call!(const_method_handle )) |
        16 => dbg_dmp!(call!(const_method_type )) |
        18 => dbg_dmp!(call!(const_invoke_dynamic )) |
        19 => dbg_dmp!(call!(const_module )) |
        20 => dbg_dmp!(call!(const_package))
    ))
);

named!(
    attribute<Attribute>,
    do_parse!(
        name_index: be_u16 >>
        length:     be_u32 >>
        info:       count!( be_u8, length as usize ) >>
        ( Attribute { name_index, info })
    )
);

named!(
    field<Field>,
    do_parse!(
        access_flags:     be_u16 >>
        name_index:       be_u16 >>
        descriptor_index: be_u16 >>
        attributes_count: be_u16 >>
        attributes:       count!( attribute, attributes_count as usize ) >>
        ( Field { access_flags, name_index, descriptor_index, attributes } )
    )
);


named!(
    method<Method>,
    do_parse!(
        access_flags:     be_u16 >>
        name_index:       be_u16 >>
        descriptor_index: be_u16 >>
        attributes_count: be_u16 >>
        attributes:       count!( attribute, attributes_count as usize ) >>
        ( Method { access_flags, name_index, descriptor_index, attributes } )
    )
);

named!(
    pub read_class_file<ClassFile>,
    dbg_dmp!(do_parse!(
        tag!(&[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8][..]) >>
        minor:              be_u16    >>
        major:              be_u16    >>
        constants_length:   be_u16    >>
        constants:          count!( constant, constants_length as usize - 1) >>
        access_flags:       be_u16    >>
        this_index:         be_u16    >>
        super_index:        be_u16    >>
        interfaces_count:   be_u16    >>
        interfaces:         count!( be_u16, interfaces_count as usize ) >>
        fields_count:       be_u16    >>
        fields:             count!( field, fields_count as usize ) >>
        methods_count:      be_u16    >>
        methods:            count!( method, methods_count as usize ) >>
        attributes_count:   be_u16    >>
        attributes:         count!( attribute, attributes_count as usize ) >>
        ( ClassFile { version: (major, minor), constants, access_flags, this_index, super_index, interfaces, fields, methods, attributes } )
    ))
);


#[cfg(test)]
mod test {
    use super::read_class_file;
    use nom::IResult;
    use java::class_file::ClassFile;

    const CLASSFILE: &'static [u8] = include_bytes!("../../../sample/HelloWorld.class");


    fn get_cf() -> ClassFile {
        read_class_file(CLASSFILE).unwrap().1
    }

    #[test]
    fn it_can_read_the_complete_class_file() {
        let cf = read_class_file(CLASSFILE).unwrap();
        match cf {
            ([], _) => (),
            _ => {
                println!("{:?}", cf);
                panic!("cannot read class file")
            }
        };
    }

    #[test]
    fn it_gets_the_version_correct() {
        let cf = get_cf();
        assert_eq!((54, 0), cf.version)
    }

    #[test]
    fn it_reads_the_correct_number_of_constants() {
        let cf = get_cf();
        assert_eq!(31, cf.constants.len())
    }

    #[test]
    fn it_gets_the_class_name_correct() {
        assert_eq!("HelloWorld", get_cf().get_class_name())
    }
}