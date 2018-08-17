use java::class_file::Method;
use java::instructions::Instruction;

pub fn disassemble<'a>(method: &Method<'a>) -> String {
    let code_block = method.get_code().unwrap();
    let instructions: Vec<Instruction> = match code_block.instructions() {
        Ok(instr) => instr,
        Err(err) => panic!("{:?}", err)
    };

    instructions.iter().map(| instruction | {
        format!("> {:?}\n", instruction)
    }).collect::<String>()

}

