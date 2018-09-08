use nom::*;

#[derive(Debug, Fail)]
pub enum ReadInstructionError<P> {
    #[fail(display = "parsing error")]
    ParsingError(Err<P>),
    #[fail(display = "parsing incomplete")]
    ParsingIncomplete,
    #[fail(display = "invalid opcode: {}", opcode)]
    InvalidOpcode { opcode: u8 },
}

macro_rules! instruction {
    ( $( $num:pat => ($size:expr): [ $($parser:tt)* ] => $name:ident ( $($a:ident: $t:ty ),* ) ),* ) => {
          #[derive(Debug, Copy, Clone)]
          pub enum Instruction {
            $(
                $name ( ( $($t),* ) )
            ),*
          }

          impl Instruction {
                pub fn get_size(&self) -> usize {
                    match self {
                        $(Instruction::$name(_) => $size),*
                    }
                }

                pub fn read_all(input: &[u8]) -> Result<Vec<Instruction>, ReadInstructionError<&[u8]>> {
                    let mut vec = Vec::new();
                    let mut remaining = &input[..];
                    loop {
                        if remaining.len() < 1 {
                            break;
                        }

                        match Instruction::read(remaining) {
                            Ok((rem, ins)) => {
                                vec.push(ins);
                                remaining = rem;
                            },
                            Err(Err::Incomplete(_)) => return Result::Err(ReadInstructionError::ParsingIncomplete),
                            Err(Err::Error(Context::Code(_, ErrorKind::Custom(42)))) => return Result::Err(ReadInstructionError::InvalidOpcode { opcode: remaining[0] }),
                            Err(err) => return Result::Err(ReadInstructionError::ParsingError(err))
                        };
                    }

                    return Result::Ok(vec);
                }

                fn read(input: &[u8]) -> IResult<&[u8], Instruction> {
                    match be_u8(input) {
                        $(
                            Ok((rem, $num)) => match do_parse!(rem, $($parser)* ) {
                                Ok((rem, ins)) => Ok((rem, Instruction::$name(ins))),
                                Err(err) => Err(err),
                            }
                        ),*,
                        Ok((_, _)) => Err(Err::Error(error_position!(input, ErrorKind::Custom(42)))),
                        Err(err) => Err(err),
                    }
                }
          }

    };
}


instruction!(
    0x00 => (1): [ () ] => NOOP(),
    0x01 => (1): [ () ] => AConstNull(),
    0x02 => (1): [ () ] => IConstm1(),
    0x03 => (1): [ () ] => IConst0(),
    0x04 => (1): [ () ] => IConst1(),
    0x05 => (1): [ () ] => IConst2(),
    0x06 => (1): [ () ] => IConst3(),
    0x07 => (1): [ () ] => IConst4(),
    0x08 => (1): [ () ] => IConst5(),
    0x09 => (1): [ () ] => LConst0(),
    0x0a => (1): [ () ] => LConst1(),
    0x0b => (1): [ () ] => FConst0(),
    0x0c => (1): [ () ] => FConst1(),
    0x0d => (1): [ () ] => FConst2(),
    0x0e => (1): [ () ] => DConst0(),
    0x0f => (1): [ () ] => DConst1(),
    0x10 => (2): [ a: be_u8  >> ( (a) ) ] => BIPush( a: u8 ),
    0x11 => (3): [ a: be_u16 >> ( (a) ) ] => SIPush( a: u16 ),
    0x12 => (2): [ a: be_u8  >> ( (a) ) ] => LDC( a: u8 ),
    0x13 => (3): [ a: be_u16 >> ( (a) ) ] => LDCW( a: u16 ),
    0x14 => (3): [ a: be_u16 >> ( (a) ) ] => LDC2W( a: u16 ),
    0x15 => (2): [ a: be_u8  >> ( (a) ) ] => ILoad( a: u8 ),
    0x16 => (2): [ a: be_u8  >> ( (a) ) ] => LLoad( a: u8 ),
    0x17 => (2): [ a: be_u8  >> ( (a) ) ] => FLoad( a: u8 ),
    0x18 => (2): [ a: be_u8  >> ( (a) ) ] => DLoad( a: u8 ),
    0x19 => (2): [ a: be_u8  >> ( (a) ) ] => ALoad( a: u8 ),
    0x1a => (1): [ () ] => ILoad0(),
    0x1b => (1): [ () ] => ILoad1(),
    0x1c => (1): [ () ] => ILoad2(),
    0x1d => (1): [ () ] => ILoad3(),
    0x1e => (1): [ () ] => LLoad0(),
    0x1f => (1): [ () ] => LLoad1(),
    0x20 => (1): [ () ] => LLoad2(),
    0x21 => (1): [ () ] => LLoad3(),
    0x22 => (1): [ () ] => FLoad0(),
    0x23 => (1): [ () ] => FLoad1(),
    0x24 => (1): [ () ] => FLoad2(),
    0x25 => (1): [ () ] => FLoad3(),
    0x26 => (1): [ () ] => DLoad0(),
    0x27 => (1): [ () ] => DLoad1(),
    0x28 => (1): [ () ] => DLoad2(),
    0x29 => (1): [ () ] => DLoad3(),
    0x2a => (1): [ () ] => ALoad0(),
    0x2b => (1): [ () ] => ALoad1(),
    0x2c => (1): [ () ] => ALoad2(),
    0x2d => (1): [ () ] => ALoad3(),
    0x2e => (1): [ () ] => IALoad(),
    0x2f => (1): [ () ] => LALoad(),
    0x30 => (1): [ () ] => FALoad(),
    0x31 => (1): [ () ] => DALoad(),
    0x32 => (1): [ () ] => AALoad(),
    0x33 => (1): [ () ] => BALoad(),
    0x34 => (1): [ () ] => CALoad(),
    0x35 => (1): [ () ] => ScALoad(),
    0x36 => (2): [ a: be_u8 >> ( ( a ) ) ] => IStore( a: u8 ),
    0x37 => (2): [ a: be_u8 >> ( ( a ) ) ] => LStore( a: u8 ),
    0x38 => (2): [ a: be_u8 >> ( ( a ) ) ] => FStore( a: u8 ),
    0x39 => (2): [ a: be_u8 >> ( ( a ) ) ] => DStore( a: u8 ),
    0x3a => (2): [ a: be_u8 >> ( ( a ) ) ] => AStore( a: u8 ),
    0x3b => (1): [ () ] => IStore0(),
    0x3c => (1): [ () ] => IStore1(),
    0x3d => (1): [ () ] => IStore2(),
    0x3e => (1): [ () ] => IStore3(),
    0x3f => (1): [ () ] => LStore0(),
    0x40 => (1): [ () ] => LStore1(),
    0x41 => (1): [ () ] => LStore2(),
    0x42 => (1): [ () ] => LStore3(),
    0x43 => (1): [ () ] => FStore0(),
    0x44 => (1): [ () ] => FStore1(),
    0x45 => (1): [ () ] => FStore2(),
    0x46 => (1): [ () ] => FStore3(),
    0x47 => (1): [ () ] => DStore0(),
    0x48 => (1): [ () ] => DStore1(),
    0x49 => (1): [ () ] => DStore2(),
    0x4a => (1): [ () ] => DStore3(),
    0x4b => (1): [ () ] => AStore0(),
    0x4c => (1): [ () ] => AStore1(),
    0x4d => (1): [ () ] => AStore2(),
    0x4e => (1): [ () ] => AStore3(),
    0x4f => (1): [ () ] => IAStore(),
    0x50 => (1): [ () ] => LAStore(),
    0x51 => (1): [ () ] => FAStore(),
    0x52 => (1): [ () ] => DAStore(),
    0x53 => (1): [ () ] => AAStore(),
    0x54 => (1): [ () ] => BAStore(),
    0x55 => (1): [ () ] => CAStore(),
    0x56 => (1): [ () ] => SAStore(),
    0x57 => (1): [ () ] => Pop(),
    0x58 => (1): [ () ] => Pop2(),
    0x59 => (1): [ () ] => Dup(),
    0x5a => (1): [ () ] => DupX1(),
    0x5b => (1): [ () ] => DupX2(),
    0x5c => (1): [ () ] => Dup2(),
    0x5d => (1): [ () ] => Dup2X1(),
    0x5e => (1): [ () ] => Dup2X2(),
    0x5f => (1): [ () ] => Swap(),
    0x60 => (1): [ () ] => IAdd(),
    0x61 => (1): [ () ] => LAdd(),
    0x62 => (1): [ () ] => FAdd(),
    0x63 => (1): [ () ] => DAdd(),
    0x64 => (1): [ () ] => ISub(),
    0x65 => (1): [ () ] => LSub(),
    0x66 => (1): [ () ] => FSub(),
    0x67 => (1): [ () ] => DSub(),
    0x68 => (1): [ () ] => IMul(),
    0x69 => (1): [ () ] => LMul(),
    0x6a => (1): [ () ] => FMul(),
    0x6b => (1): [ () ] => DMul(),
    0x6c => (1): [ () ] => IDiv(),
    0x6d => (1): [ () ] => LDiv(),
    0x6e => (1): [ () ] => FDiv(),
    0x6f => (1): [ () ] => DDiv(),
    0x70 => (1): [ () ] => IRem(),
    0x71 => (1): [ () ] => LRem(),
    0x72 => (1): [ () ] => FRem(),
    0x73 => (1): [ () ] => DRem(),
    0x74 => (1): [ () ] => INeg(),
    0x75 => (1): [ () ] => LNeg(),
    0x76 => (1): [ () ] => FNeg(),
    0x77 => (1): [ () ] => DNeg(),
    0x78 => (1): [ () ] => IShl(),
    0x79 => (1): [ () ] => LShl(),
    0x7a => (1): [ () ] => IShr(),
    0x7b => (1): [ () ] => LShr(),
    0x7c => (1): [ () ] => IUSHR(),
    0x7d => (1): [ () ] => LUSHR(),
    0x7e => (1): [ () ] => IAnd(),
    0x7f => (1): [ () ] => LAnd(),
    0x80 => (1): [ () ] => IOr(),
    0x81 => (1): [ () ] => LOr(),
    0x82 => (1): [ () ] => IXor(),
    0x83 => (1): [ () ] => LXor(),
    0x84 => (3): [ a: be_u8 >> b: be_u8 >> ( ( a, b ) ) ] => IInc( a: u8, b: u8 ),
    0x85 => (1): [ () ] => I2L(),
    0x86 => (1): [ () ] => I2F(),
    0x87 => (1): [ () ] => I2D(),
    0x88 => (1): [ () ] => L2I(),
    0x89 => (1): [ () ] => L2F(),
    0x8a => (1): [ () ] => L2D(),
    0x8b => (1): [ () ] => F2I(),
    0x8c => (1): [ () ] => F2L(),
    0x8d => (1): [ () ] => F2D(),
    0x8e => (1): [ () ] => D2I(),
    0x8f => (1): [ () ] => D2L(),
    0x90 => (1): [ () ] => D2F(),
    0x91 => (1): [ () ] => I2B(),
    0x92 => (1): [ () ] => I2C(),
    0x93 => (1): [ () ] => I2S(),
    0x94 => (1): [ () ] => LCmp(),
    0x95 => (1): [ () ] => FCmpL(),
    0x96 => (1): [ () ] => FCmpG(),
    0x97 => (1): [ () ] => DCmpL(),
    0x98 => (1): [ () ] => DCmpG(),
    0x99 => (3): [ a: be_i16 >> ( ( a ) ) ] => Ifeq( a: i16 ),
    0x9a => (3): [ a: be_i16 >> ( ( a ) ) ] => Ifne( a: i16 ),
    0x9b => (3): [ a: be_i16 >> ( ( a ) ) ] => Iflt( a: i16 ),
    0x9c => (3): [ a: be_i16 >> ( ( a ) ) ] => Ifge( a: i16 ),
    0x9d => (3): [ a: be_i16 >> ( ( a ) ) ] => Ifgt( a: i16 ),
    0x9e => (3): [ a: be_i16 >> ( ( a ) ) ] => Ifle( a: i16 ),
    0x9f => (3): [ a: be_i16 >> ( ( a ) ) ] => IfICmpEQ( a: i16 ),
    0xa0 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfICmpNE( a: i16 ),
    0xa1 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfICmpLT( a: i16 ),
    0xa2 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfICmpGE( a: i16 ),
    0xa3 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfICmpGT( a: i16 ),
    0xa4 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfICmpLE( a: i16 ),
    0xa5 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfACmpEQ( a: i16 ),
    0xa6 => (3): [ a: be_i16 >> ( ( a ) ) ] => IfACmpNE( a: i16 ),
    0xa7 => (3): [ a: be_i16 >> ( ( a ) ) ] => Goto( a: i16 ),
    0xa8 => (3): [ a: be_i16 >> ( ( a ) ) ] => JSR( a: i16 ),
    0xa9 => (2): [ a: be_u8  >> ( ( a ) ) ] => Ret( a: u8 ),
    0xaa => (2): [ a: be_u8  >> ( ( a ) ) ] => TableSwitch( a: u8 ),
    0xab => (9): [ a: be_u64 >> b: be_u64 >> ( ( a, b ) ) ] => LookupSwitch( a: u64, b: u64 ),   //TODO: this is not entirely correct, the length is not fixed
    0xac => (1): [ () ] => IReturn(),
    0xad => (1): [ () ] => LReturn(),
    0xae => (1): [ () ] => FReturn(),
    0xaf => (1): [ () ] => DReturn(),
    0xb0 => (1): [ () ] => AReturn(),
    0xb1 => (1): [ () ] => Return(),
    0xb2 => (3): [ a: be_u16 >> ( (a) ) ] => GetStatic( a: u16 ),
    0xb3 => (3): [ a: be_u16 >> ( ( a ) ) ] => PutStatic( a: u16 ),
    0xb4 => (3): [ a: be_u16 >> ( ( a ) ) ] => GetField( a: u16 ),
    0xb5 => (3): [ a: be_u16 >> ( ( a ) ) ] => PutField( a: u16 ),
    0xb6 => (3): [ a: be_u16 >> ( ( a ) ) ] => InvokeVirtual( a: u16 ),
    0xb7 => (3): [ a: be_u16 >> ( ( a ) ) ] => InvokeSpecial( a: u16 ),
    0xb8 => (3): [ a: be_u16 >> ( ( a ) ) ] => InvokeStatic( a: u16 ),
    0xb9 => (5): [ a: be_u64 >> ( ( a ) ) ] => InvokeInterface( a: u64 ),
    0xba => (5): [ a: be_u64 >> ( ( a ) ) ] => InvokeDynamic( a: u64),
    0xbb => (3): [ a: be_u16 >> ( ( a ) ) ] => New( a: u16 ),
    0xbc => (2): [ a: be_u8 >> ( ( a ) ) ] => NewArray( a: u8 ),
    0xbd => (3): [ a: be_u16 >> ( ( a ) ) ] => AAewArray( a: u16 ),
    0xbe => (1): [ () ] => ArrayLength(),
    0xbf => (1): [ () ] => AThrow(),
    0xc0 => (3): [ a: be_u16 >> ( ( a ) ) ] => CheckCast( a: u16 ),
    0xc1 => (3): [ a: be_u16 >> ( ( a ) ) ] => InstanceOf( a: u16 ),
    0xc2 => (1): [ () ] => MonitorEnter(),
    0xc3 => (1): [ () ] => MonitorExit(),
    0xc4 => (1): [ a: be_u16 >> b: be_u8 >> ( ( a, b ) ) ] => Wide( a: u16, b: u8),
    0xc5 => (1): [ a: be_u16 >> b: be_u8 >> ( ( a, b ) ) ] => MultianeWArray( a: u16, b: u8),
    0xc6 => (3): [ a: be_u16 >> ( ( a ) ) ] => IfNull( a: u16 ),
    0xc7 => (3): [ a: be_u16 >> ( ( a ) ) ] => IfNonNull( a: u16 ),
    0xc8 => (5): [ a: be_u64 >> ( ( a ) ) ] => GotoW( a: u64),
    0xc9 => (5): [ a: be_u64 >> ( ( a ) ) ] => JSRW( a: u64),
    0xca => (1): [ () ] => Breakpoint(),
    0xfe => (1): [ () ] => ImpDep1(),
    0xff => (1): [ () ] => ImpDep2()
);