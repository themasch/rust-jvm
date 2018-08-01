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
    ( $( $num:pat => [ $($parser:tt)* ] => $name:ident ( $($a:ident: $t:ty ),* ) ),* ) => {
          #[derive(Debug)]
          pub enum Instruction {
            $(
                $name ( ( $($t),* ) )
            ),*
          }

          impl Instruction {
                pub fn read_all(input: &[u8]) -> Result<Vec<Instruction>, ReadInstructionError<&[u8]>> {
                    let mut vec = Vec::new();
                    let mut remaining = &input[..];
                    loop {
                        if remaining.len() < 1 {
                            break;
                        }

                        match Instruction::read(remaining) {
                            IResult::Done(rem, ins) => {
                                vec.push(ins);
                                remaining = rem;
                            },
                            IResult::Error(Err::Code(ErrorKind::Custom(42))) => return Result::Err(ReadInstructionError::InvalidOpcode { opcode: remaining[0] }),
                            IResult::Error(err) => return Result::Err(ReadInstructionError::ParsingError(err)),
                            IResult::Incomplete(_) => return Result::Err(ReadInstructionError::ParsingIncomplete)
                        };
                    }

                    return Result::Ok(vec);
                }

                fn read(input: &[u8]) -> IResult<&[u8], Instruction> {
                    match be_u8(input) {
                        $(
                            IResult::Done(rem, $num) => match do_parse!(rem, $($parser)* ) {
                                IResult::Done(rem, ins) => IResult::Done(rem, Instruction::$name(ins)),
                                IResult::Error(err) => IResult::Error(err),
                                IResult::Incomplete(_) => panic!("incomplete code")
                            }
                        ),*,
                        IResult::Done(_, _) => IResult::Error(Err::Code(ErrorKind::Custom(42))),
                        IResult::Error(err) => IResult::Error(err),
                        IResult::Incomplete(_) => panic!("incomplete code")
                    }
                }
          }

    };
}


instruction!(
    0x00 => [ () ] => NOOP(),
    0x01 => [ () ] => AConstNull(),
    0x02 => [ () ] => IConstm1(),
    0x03 => [ () ] => IConst0(),
    0x04 => [ () ] => IConst1(),
    0x05 => [ () ] => IConst2(),
    0x06 => [ () ] => IConst3(),
    0x07 => [ () ] => IConst4(),
    0x08 => [ () ] => IConst5(),
    0x09 => [ () ] => LConst0(),
    0x0a => [ () ] => LConst1(),
    0x0b => [ () ] => FConst0(),
    0x0c => [ () ] => FConst1(),
    0x0d => [ () ] => FConst2(),
    0x0e => [ () ] => DConst0(),
    0x0f => [ () ] => DConst1(),
    0x10 => [ a: be_u8  >> ( (a) ) ] => BIPush( a: u8 ),
    0x11 => [ a: be_u16 >> ( (a) ) ] => SIPush( a: u16 ),
    0x12 => [ a: be_u8  >> ( (a) ) ] => LDC( a: u8 ),
    0x13 => [ a: be_u16 >> ( (a) ) ] => LDCW( a: u16 ),
    0x14 => [ a: be_u16 >> ( (a) ) ] => LDC2W( a: u16 ),
    0x15 => [ a: be_u8  >> ( (a) ) ] => ILoad( a: u8 ),
    0x16 => [ a: be_u8  >> ( (a) ) ] => LLoad( a: u8 ),
    0x17 => [ a: be_u8  >> ( (a) ) ] => FLoad( a: u8 ),
    0x18 => [ a: be_u8  >> ( (a) ) ] => DLoad( a: u8 ),
    0x19 => [ a: be_u8  >> ( (a) ) ] => ALoad( a: u8 ),
    0x1a => [ () ] => ILoad0(),
    0x1b => [ () ] => ILoad1(),
    0x1c => [ () ] => ILoad2(),
    0x1d => [ () ] => ILoad3(),
    0x1e => [ () ] => LLoad0(),
    0x1f => [ () ] => LLoad1(),
    0x20 => [ () ] => LLoad2(),
    0x21 => [ () ] => LLoad3(),
    0x22 => [ () ] => FLoad0(),
    0x23 => [ () ] => FLoad1(),
    0x24 => [ () ] => FLoad2(),
    0x25 => [ () ] => FLoad3(),
    0x26 => [ () ] => DLoad0(),
    0x27 => [ () ] => DLoad1(),
    0x28 => [ () ] => DLoad2(),
    0x29 => [ () ] => DLoad3(),
    0x2a => [ () ] => ALoad0(),
    0x2b => [ () ] => ALoad1(),
    0x2c => [ () ] => ALoad2(),
    0x2d => [ () ] => ALoad3(),
    0x2e => [ () ] => IALoad(),
    0x2f => [ () ] => LALoad(),
    0x30 => [ () ] => FALoad(),
    0x31 => [ () ] => DALoad(),
    0x32 => [ () ] => AALoad(),
    0x33 => [ () ] => BALoad(),
    0x34 => [ () ] => CALoad(),
    0x35 => [ () ] => ScALoad(),
    0x36 => [ a: be_u8 >> ( ( a ) ) ] => IStore( a: u8 ),
    0x37 => [ a: be_u8 >> ( ( a ) ) ] => LStore( a: u8 ),
    0x38 => [ a: be_u8 >> ( ( a ) ) ] => FStore( a: u8 ),
    0x39 => [ a: be_u8 >> ( ( a ) ) ] => DStore( a: u8 ),
    0x3a => [ a: be_u8 >> ( ( a ) ) ] => AStore( a: u8 ),
    0x3b => [ () ] => IStore0(),
    0x3c => [ () ] => IStore1(),
    0x3d => [ () ] => IStore2(),
    0x3e => [ () ] => IStore3(),
    0x3f => [ () ] => LStore0(),
    0x40 => [ () ] => LStore1(),
    0x41 => [ () ] => LStore2(),
    0x42 => [ () ] => LStore3(),
    0x43 => [ () ] => FStore0(),
    0x44 => [ () ] => FStore1(),
    0x45 => [ () ] => FStore2(),
    0x46 => [ () ] => FStore3(),
    0x47 => [ () ] => DStore0(),
    0x48 => [ () ] => DStore1(),
    0x49 => [ () ] => DStore2(),
    0x4a => [ () ] => DStore3(),
    0x4b => [ () ] => AStore0(),
    0x4c => [ () ] => AStore1(),
    0x4d => [ () ] => AStore2(),
    0x4e => [ () ] => AStore3(),
    0x4f => [ () ] => IAStore(),
    0x50 => [ () ] => LAStore(),
    0x51 => [ () ] => FAStore(),
    0x52 => [ () ] => DAStore(),
    0x53 => [ () ] => AAStore(),
    0x54 => [ () ] => BAStore(),
    0x55 => [ () ] => CAStore(),
    0x56 => [ () ] => SAStore(),
    0x57 => [ () ] => Pop(),
    0x58 => [ () ] => Pop2(),
    0x59 => [ () ] => Dup(),
    0x5a => [ () ] => DupX1(),
    0x5b => [ () ] => DupX2(),
    0x5c => [ () ] => Dup2(),
    0x5d => [ () ] => Dup2X1(),
    0x5e => [ () ] => Dup2X2(),
    0x5f => [ () ] => Swap(),
    0x60 => [ () ] => IAdd(),
    0x61 => [ () ] => LAdd(),
    0x62 => [ () ] => FAdd(),
    0x63 => [ () ] => DAdd(),
    0x64 => [ () ] => ISub(),
    0x65 => [ () ] => LSub(),
    0x66 => [ () ] => FSub(),
    0x67 => [ () ] => DSub(),
    0x68 => [ () ] => IMul(),
    0x69 => [ () ] => LMul(),
    0x6a => [ () ] => FMul(),
    0x6b => [ () ] => DMul(),
    0x6c => [ () ] => IDiv(),
    0x6d => [ () ] => LDiv(),
    0x6e => [ () ] => FDiv(),
    0x6f => [ () ] => DDiv(),
    0x70 => [ () ] => IRem(),
    0x71 => [ () ] => LRem(),
    0x72 => [ () ] => FRem(),
    0x73 => [ () ] => DRem(),
    0x74 => [ () ] => INeg(),
    0x75 => [ () ] => LNeg(),
    0x76 => [ () ] => FNeg(),
    0x77 => [ () ] => DNeg(),
    0x78 => [ () ] => IShl(),
    0x79 => [ () ] => LShl(),
    0x7a => [ () ] => IShr(),
    0x7b => [ () ] => LShr(),
    0x7c => [ () ] => IUSHR(),
    0x7d => [ () ] => LUSHR(),
    0x7e => [ () ] => IAnd(),
    0x7f => [ () ] => LAnd(),
    0x80 => [ () ] => IOr(),
    0x81 => [ () ] => LOr(),
    0x82 => [ () ] => IXor(),
    0x83 => [ () ] => LXor(),
    0x84 => [ a: be_u16 >> ( ( a ) ) ] => IInc( a: u16 ),
    0x85 => [ () ] => I2L(),
    0x86 => [ () ] => I2F(),
    0x87 => [ () ] => I2D(),
    0x88 => [ () ] => L2I(),
    0x89 => [ () ] => L2F(),
    0x8a => [ () ] => L2D(),
    0x8b => [ () ] => F2I(),
    0x8c => [ () ] => F2L(),
    0x8d => [ () ] => F2D(),
    0x8e => [ () ] => D2I(),
    0x8f => [ () ] => D2L(),
    0x90 => [ () ] => D2F(),
    0x91 => [ () ] => I2B(),
    0x92 => [ () ] => I2C(),
    0x93 => [ () ] => I2S(),
    0x94 => [ () ] => LCmp(),
    0x95 => [ () ] => FCmpL(),
    0x96 => [ () ] => FCmpG(),
    0x97 => [ () ] => DCmpL(),
    0x98 => [ () ] => DCmpG(),
    0x99 => [ a: be_u16 >> ( ( a ) ) ] => Ifeq( a: u16 ),
    0x9a => [ a: be_u16 >> ( ( a ) ) ] => Ifne( a: u16 ),
    0x9b => [ a: be_u16 >> ( ( a ) ) ] => Iflt( a: u16 ),
    0x9c => [ a: be_u16 >> ( ( a ) ) ] => Ifge( a: u16 ),
    0x9d => [ a: be_u16 >> ( ( a ) ) ] => Ifgt( a: u16 ),
    0x9e => [ a: be_u16 >> ( ( a ) ) ] => Ifle( a: u16 ),
    0x9f => [ a: be_u16 >> ( ( a ) ) ] => IfICmpEQ( a: u16 ),
    0xa0 => [ a: be_u16 >> ( ( a ) ) ] => IfICmpNE( a: u16 ),
    0xa1 => [ a: be_u16 >> ( ( a ) ) ] => IfICmpLT( a: u16 ),
    0xa2 => [ a: be_u16 >> ( ( a ) ) ] => IfICmpGE( a: u16 ),
    0xa3 => [ a: be_u16 >> ( ( a ) ) ] => IfICmpGT( a: u16 ),
    0xa4 => [ a: be_u16 >> ( ( a ) ) ] => IfICmpLE( a: u16 ),
    0xa5 => [ a: be_u16 >> ( ( a ) ) ] => IfACmpEQ( a: u16 ),
    0xa6 => [ a: be_u16 >> ( ( a ) ) ] => IfACmpNE( a: u16 ),
    0xa7 => [ a: be_u16 >> ( ( a ) ) ] => Goto( a: u16 ),
    0xa8 => [ a: be_u16 >> ( ( a ) ) ] => JSR( a: u16 ),
    0xa9 => [ a: be_u8  >> ( ( a ) ) ] => Ret( a: u8 ),
    0xaa => [ a: be_u8  >> ( ( a ) ) ] => TableSwitch( a: u8 ),
    0xab => [ a: be_u64 >> b: be_u64 >> ( ( a, b ) ) ] => LookupSwitch( a: u64, b: u64 ),
    0xac => [ () ] => IReturn(),
    0xad => [ () ] => LReturn(),
    0xae => [ () ] => FReturn(),
    0xaf => [ () ] => DReturn(),
    0xb0 => [ () ] => AReturn(),
    0xb1 => [ () ] => Return(),
    0xb2 => [ a: be_u16 >> ( (a) ) ] => GetStatic( a: u16 ),
    0xb3 => [ a: be_u16 >> ( ( a ) ) ] => PutStatic( a: u16 ),
    0xb4 => [ a: be_u16 >> ( ( a ) ) ] => GetField( a: u16 ),
    0xb5 => [ a: be_u16 >> ( ( a ) ) ] => PutField( a: u16 ),
    0xb6 => [ a: be_u16 >> ( ( a ) ) ] => InvokeVirtual( a: u16 ),
    0xb7 => [ a: be_u16 >> ( ( a ) ) ] => InvokeSpecial( a: u16 ),
    0xb8 => [ a: be_u16 >> ( ( a ) ) ] => InvokeStatic( a: u16 ),
    0xb9 => [ a: be_u64 >> ( ( a ) ) ] => InvokeInterface( a: u64 ),
    0xba => [ a: be_u64 >> ( ( a ) ) ] => InvokeDynamic( a: u64),
    0xbb => [ a: be_u16 >> ( ( a ) ) ] => New( a: u16 ),
    0xbc => [ a: be_u8 >> ( ( a ) ) ] => NewArray( a: u8 ),
    0xbd => [ a: be_u16 >> ( ( a ) ) ] => AAewArray( a: u16 ),
    0xbe => [ () ] => ArrayLength(),
    0xbf => [ () ] => AThrow(),
    0xc0 => [ a: be_u16 >> ( ( a ) ) ] => CheckCast( a: u16 ),
    0xc1 => [ a: be_u16 >> ( ( a ) ) ] => InstanceOf( a: u16 ),
    0xc2 => [ () ] => MonitorEnter(),
    0xc3 => [ () ] => MonitorExit(),
    0xc4 => [ a: be_u16 >> b: be_u8 >> ( ( a, b ) ) ] => Wide( a: u16, b: u8),
    0xc5 => [ a: be_u16 >> b: be_u8 >> ( ( a, b ) ) ] => MultianeWArray( a: u16, b: u8),
    0xc6 => [ a: be_u16 >> ( ( a ) ) ] => IfNull( a: u16 ),
    0xc7 => [ a: be_u16 >> ( ( a ) ) ] => IfNonNull( a: u16 ),
    0xc8 => [ a: be_u64 >> ( ( a ) ) ] => GotoW( a: u64),
    0xc9 => [ a: be_u64 >> ( ( a ) ) ] => JSRW( a: u64),
    0xca => [ () ] => Breakpoint(),
    0xfe => [ () ] => ImpDep1(),
    0xff => [ () ] => ImpDep2()
);