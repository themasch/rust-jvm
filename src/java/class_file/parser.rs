use nom::*;

use super::*;

use std::string::String;
use std::str::from_utf8;

named!(
    parse_type<&[u8], ValueType>,
    dbg_dmp!(switch!(take!(1),
        b"L" => do_parse!( tn: map_res!(take_until!(";"), from_utf8) >> (ValueType::Object(String::from(tn)))) |
        b"I" => value!(ValueType::Integer) |
        b"V" => value!(ValueType::Void) |
        b"[" => do_parse!( ele: parse_type >> (ValueType::Array(Box::new(ele)))) |
        _ => value!(ValueType::Void)
    ))
);

pub fn param_list(input: &[u8]) -> IResult<&[u8], Vec<ValueType>> {
    if input[0] != b'(' {
        return Err(Err::Incomplete(Needed::Size(2)));
    }

    let mut input = &input[1..];
    let mut vec = Vec::new();
    loop {
        if input[0] == b')' {
            return Ok((&input[1..], vec));
        }

        match parse_type(input) {
            Ok((rem, typ)) => {
                vec.push(typ);
                input = rem;
            },
            Err(err) => return Err(err)
        }
    }.clone()
}

named!(
    pub method_desc<&[u8], (Vec<ValueType>, ValueType)>,
    dbg_dmp!(tuple!(
        param_list,
        parse_type
    ))
);

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
    do_parse!(bytes: length_data!(be_u16) >> ( ConstantType::Utf8 { value: from_utf8(bytes).unwrap() } ) )
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
    exception_table<(u16, u16, u16, u16)>,
    do_parse!(
        start_pc: be_u16   >>
        end_pc: be_u16     >>
        handler_pc: be_u16 >>
        catch_type: be_u16 >>
        (start_pc, end_pc, handler_pc, catch_type)
    )
);

named!(
    line_number_table<Attribute>,
    do_parse!(
        length: be_u32 >>
        line_numbers: length_count!(
            be_u16,
            do_parse!(
                start: be_u16 >>
                line: be_u16 >>
                (start, line)
            )
        ) >>
        (
            Attribute::LineNumberTable(line_numbers.to_vec())
        )
    )
);

fn select_attribute<'t, 'a>(input: &'t [u8], name: &str, constants: &'a Vec<ConstantType<'a>>) -> IResult<&'t [u8], Attribute<'t>> {
    match name {
        "LineNumberTable" => {
            match line_number_table(input) {
                Ok((rem, line_numbers)) => {
                    Ok((&rem, line_numbers))
                }
                Err(err) => return Err(err)
            }
        }
        "Code" => {
            match do_parse!( input,
                    be_u32 >>
                    max_stack: be_u16 >>
                    max_locals: be_u16 >>
                    code: length_data!( be_u32 ) >>
                    exception_table: length_count!( be_u16, exception_table ) >>
                    attributes: length_count!( be_u16, call!(attribute, &constants)) >>
                    (
                        Attribute::CodeAttribute( CodeBlock { max_stack, max_locals, code: code.to_vec(), attributes } )
                    )
                ) {
                Ok((rem, attribute)) => Ok((&rem, attribute)),
                Err(err) => return Err(err)
            }
        }
        _ => {
            let nm = String::from(name);
            match be_u32(input) {
                Ok((rem, length)) => {
                    Ok((&rem[(length as usize)..], Attribute::GenericAttribute { name: nm, info: &rem[0..(length as usize)] }))
                }
                Err(err) => return Err(err)
            }
        }
    }
}

fn attribute<'t, 'a>(input: &'t [u8], constants: &'a Vec<ConstantType<'a>>) -> IResult<&'t [u8], Attribute<'t>> {
    let idx_res = be_u16(input);
    match idx_res {
        Ok((remaining, index)) => {
            match constants.get(index as usize - 1) {
                Some(ConstantType::Utf8 { value: ref name }) => {
                    select_attribute(remaining, name, constants)
                }
                _ => {
                    Err(Err::Error(error_position!(remaining, ErrorKind::Custom(1))))
                }
            }
        }
        Err(err) => return Err(err)
    }
}

named_args!(
    field<'a>(constants: &'a Vec<ConstantType<'this_is_probably_unique_i_hope_please>>)<Field<'this_is_probably_unique_i_hope_please>>,
    do_parse!(
        access_flags:     be_u16 >>
        name_index:       be_u16 >>
        descriptor_index: be_u16 >>
        attributes_count: be_u16 >>
        attributes:       count!( call!(attribute, constants), attributes_count as usize ) >>
        ( Field { access_flags, name_index, descriptor_index, attributes } )
    )
);


named_args!(
    method<'a>(constants: &'a Vec<ConstantType<'this_is_probably_unique_i_hope_please>>)<Method<'this_is_probably_unique_i_hope_please>>,
    do_parse!(
        access_flags:     be_u16 >>
        name_index:       be_u16 >>
        descriptor_index: be_u16 >>
        attributes_count: be_u16 >>
        attributes:       count!( call!(attribute, constants), attributes_count as usize ) >>
        ( Method {
            access_flags,
            name: match constants.get(usize::from(name_index - 1)).unwrap() {
                ConstantType::Utf8 { value: str } => str,
                _ => panic!("wrong constant type")
            },
            descriptor: match constants.get(usize::from(descriptor_index - 1)).unwrap() {
                ConstantType::Utf8 { value: str } => str,
                _ => panic!("wrong constant type")
            },
            attributes
          }
        )
    )
);

named!(
    pub read_class_file<ClassFile>,
    dbg!(do_parse!(
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
        fields:             count!( call!(field, &constants), fields_count as usize ) >>
        methods_count:      be_u16    >>
        methods:            count!( call!(method, &constants), methods_count as usize ) >>
        attributes_count:   be_u16    >>
        attributes:         count!( call!(attribute, &constants), attributes_count as usize ) >>
        ( ClassFile { version: (major, minor), constants, access_flags, this_index, super_index, interfaces, fields, methods, attributes } )
    ))
);


#[cfg(test)]
mod test {
    use super::read_class_file;
    use java::class_file::ClassFile;

    const CLASSFILE: &'static [u8] = include_bytes!("../../../sample/HelloWorld.class");
    const DEMOCLASS: &'static [u8] = include_bytes!("../../../sample/DemoClass.class");


    fn get_cf<'a>() -> ClassFile<'a> {
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


    ///////// method descriptor
    use super::*;
    use nom::*;
    use java::class_file::ValueType;

    #[test]
    fn test_param_list_int_int() {
        let res = param_list(b"(II)");
        let vec = vec![ValueType::Integer, ValueType::Integer];

        match res {
            Ok((_, rvec)) => assert_eq!(rvec, vec),
            _ => assert_eq!(true, false)
        };
    }

    #[test]
    fn test_method_desc_void() {
        let input = b"()V";
        let vec = vec![];
        let retvalue = ValueType::Void;
        match method_desc(b"()V") {
            Ok((_, rvec)) => assert_eq!(rvec, (vec, retvalue)),
            _ => assert_eq!(true, false)
        };
    }
}