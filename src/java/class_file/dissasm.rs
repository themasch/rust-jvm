use java::class_file::Method;
use java::instructions::Instruction;
use nom::*;

pub fn dissassemble<'a>(method: &Method<'a>) -> String {
    let instructions: Vec<Instruction> = match method.get_code().unwrap().instructions() {
        Ok(instr) => instr,
        Err(err) => panic!("{:?}", err)
    };

    return instructions.iter().map(| instruction | {
        format!("> {:?}\n", instruction)
    }).collect::<String>();

    //return format!("{:x?}", code.code);
}

