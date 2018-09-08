use java::class_file::Method;
use std::collections::HashMap;
use java::class_file::ClassFile;
use std::path::PathBuf;
use std::sync::Arc;
use java::class_file::ConstantType;
use java::class_file::ValueType;


use java::instructions::Instruction;

/// these type of errors should not happen at all.
/// Triggering one of these means the jvm is probably buggy since the compiler should prevent these.
/// This is for stuff like "we tried to pop the stack but it was empty" or "i need to load an int,
/// but theres a string on the stack"….
///
/// we might trigger something like this when a class file does not contain the expected methods.
/// this is something the compiler cannot prevent since the user could just swap out the class file.
#[derive(Debug, Fail)]
pub enum RuntimeError {
    #[fail(display = "runtime error: {}", message)]
    GenericError { message: String },
    #[fail(display = "runtime error: top of stack had the wrong type. expected: {}", expected)]
    StackType { expected: String },
    #[fail(display = "runtime error: stack poped when empty")]
    EmptyStack,
    #[fail(display = "runtime error: method not found")]
    MethodNotFound,
    #[fail(display = "runtime error: no such variable ")]
    VariableOutOfScope,
    #[fail(display = "runtime error: variable at index {} has the wrong type. expected: {}", offset, expected)]
    VariableType { expected: String, offset: usize },
}

#[derive(Debug)]
enum LocalVariable {
    None,
    Null,
    Integer(i64),
}

#[derive(Debug, Eq, PartialEq)]
enum StackValue {
    None,
    Null,
    Integer(i64),
}

#[derive(Debug)]
struct StackFrame {
    local_variables: Vec<LocalVariable>,
    stack: Vec<StackValue>,
}

impl StackFrame {
    fn create(var_count: usize, stack_size: usize) -> StackFrame {
        StackFrame {
            local_variables: StackFrame::init_variables(var_count),
            stack: Vec::with_capacity(stack_size),
        }
    }

    fn init_variables(size: usize) -> Vec<LocalVariable> {
        let mut vec = Vec::with_capacity(size);
        for i in 0..size {
            vec.push(LocalVariable::None);
        }

        vec
    }

    /// creates a new `StackFrame` for a given method.
    /// also inits the local variables with the given list of variables
    fn for_method(method: &Method, mut variables: Vec<LocalVariable>) -> StackFrame {
        let locals = usize::from(method.get_code().unwrap().max_locals);
        let stack = usize::from(method.get_code().unwrap().max_stack);

        let mut stack = StackFrame::create(locals, stack);
        for i in 0..variables.len() {
            stack.local_variables[i] = variables.remove(0);
        }

        stack
    }

    fn get_variable_mut(&mut self, index: usize) -> Option<&mut LocalVariable> {
        self.local_variables.get_mut(index)
    }

    fn get_variable(&mut self, index: usize) -> Option<&LocalVariable> {
        self.local_variables.get(index)
    }

    fn set_variable(&mut self, index: usize, var: LocalVariable) {
        self.local_variables.insert(index, var)
    }

    fn pop_stack(&mut self) -> Option<StackValue> {
        self.stack.pop()
    }

    fn push_stack(&mut self, value: StackValue) {
        self.stack.push(value)
    }
}

/// this might be terrible named (it is).
/// it contains information on how the interpreter should continue after an instruction
/// has been executed
enum InstructionResult {
    Continue,
    Goto(i32),
    Return(Option<StackValue>),
}

pub struct Context<'b> {
    return_value: Option<StackValue>,
    class: Arc<ClassFile<'b>>,
}

pub struct Runtime<'a> {
    classes: HashMap<String, Arc<ClassFile<'a>>>,
    classpath: Vec<PathBuf>,
    main_class: String,
    class_index_map: HashMap<String, HashMap<usize, String>>,
}

impl<'a> Runtime<'a> {
    pub fn create(main_class: ClassFile<'a>) -> Runtime<'a> {
        let name = String::from(main_class.get_class_name());
        let mut rt = Runtime {
            classes: HashMap::new(),
            classpath: vec![PathBuf::from(".")],
            class_index_map: HashMap::new(),
            main_class: name,
        };

        rt.load_class(main_class);

        return rt;
    }

    fn build_class_index_map(class: &ClassFile<'a>) -> HashMap<usize, String> {
        let cla_idx_map = class.constants
            .iter()
            .filter_map(|mref| match mref {
                ConstantType::MethodRef { class_index: cli, .. } => Some(cli),
                _ => None
            })
            .filter_map(|class_index| {
                match class.get_constant(*class_index) {
                    Some(ConstantType::Class { name_index: idx }) => Some((class_index, idx)),
                    _ => None
                }
            })
            .filter_map(|(class_index, name_index)| {
                match class.get_constant(*name_index) {
                    Some(ConstantType::Utf8 { value }) => Some((class_index, value.clone())),
                    _ => None
                }
            });

        let mut map = HashMap::new();
        for (class_index, name) in cla_idx_map {
            map.insert(usize::from(*class_index), String::from(name));
        }

        return map;
    }

    pub fn load_class(&mut self, class: ClassFile<'a>) {
        let map = Runtime::build_class_index_map(&class);
        let name = String::from(class.get_class_name());
        self.class_index_map.insert(name.clone(), map);
        self.classes.insert(name, Arc::new(class));
    }

    pub fn run(&mut self) {
        let class = self.classes.get(&self.main_class).expect("no main class loaded").clone();
        let method = class.methods.iter().find(|method| method.name.eq("main"));
        if method.is_none() {
            eprintln!("Class {} does not have a main method", class.get_class_name());
            return;
        }

        match self.run_method(method.unwrap(), class.clone(), vec![]) {
            Ok(ret) => println!("main return value: {:?}", ret),
            Err(err) => eprintln!("runtime error: {:?}", err)
        }
    }

    #[cfg(test)]
    pub fn exec_method_on_main(&mut self, method_name: &str) -> Result<Option<StackValue>, RuntimeError> {
        let class = self.classes.get(&self.main_class).expect("no main class loaded").clone();
        let method = class.methods.iter().find(|method| method.name.eq(method_name));
        if method.is_none() {
            return Err(RuntimeError::GenericError { message: format!("Class {} does not have a main method", class.get_class_name()) });
        }

        return self.run_method(method.unwrap(), class.clone(), vec![]);
    }

    /// stores the top stack value into the local variable at `offset` as an integer
    /// since our stack is typed, we only do this when the type of the uppermost stack value is integer, too.
    fn exec_istore(stack_frame: &mut StackFrame, offset: usize) -> Result<(), RuntimeError> {
        match stack_frame.pop_stack() {
            Some(StackValue::Integer(intvalue)) => {
                stack_frame.set_variable(offset, LocalVariable::Integer(intvalue));
                Ok(())
            }
            _ => {
                Err(RuntimeError::GenericError { message: format!("stack value at index {} is not an integer", offset) })
            }
        }
    }

    /// loads an integer from local variable `offset` onto the stack.
    /// fails if
    ///  - the local variable is not an integer
    ///  - the local variable is not even defined
    ///  - the local variable is out of scope
    fn exec_iload(stack_frame: &mut StackFrame, offset: usize) -> Result<(), RuntimeError> {
        let intvalue = match stack_frame.get_variable(offset) {
            Some(LocalVariable::Integer(intvalue)) => {
                *intvalue
            }
            Some(LocalVariable::None) => return Err(RuntimeError::GenericError { message: format!("local variable at index {} is not defined", offset) }),
            Some(_) => return Err(RuntimeError::GenericError { message: format!("local variable at index {} is not an integer", offset) }),
            None => return Err(RuntimeError::GenericError { message: format!("stack value at index {} is out of range", offset) })
        };

        stack_frame.push_stack(StackValue::Integer(intvalue));
        Ok(())
    }

    fn exec(&mut self, instruction: &Instruction, mut stack_frame: &mut StackFrame, context: &mut Context<'a>) -> Result<InstructionResult, RuntimeError> {
        // since most of the instructions just operate on the StackFrame, and the return value
        // it might be useful to move these implementations somewhere else.
        // although some instructions actually need more knownledge about the context, like the
        // current class, loaded classes, change the next instruction etc.
        match instruction {
            //00
            Instruction::IConstm1(()) => stack_frame.push_stack(StackValue::Integer(-1)),
            Instruction::IConst0(()) => stack_frame.push_stack(StackValue::Integer(0)),
            Instruction::IConst1(()) => stack_frame.push_stack(StackValue::Integer(1)),
            Instruction::IConst2(()) => stack_frame.push_stack(StackValue::Integer(2)),
            Instruction::IConst3(()) => stack_frame.push_stack(StackValue::Integer(3)),
            Instruction::IConst4(()) => stack_frame.push_stack(StackValue::Integer(4)),
            Instruction::IConst5(()) => stack_frame.push_stack(StackValue::Integer(5)),
            // 10...
            Instruction::BIPush(value) =>
                stack_frame.push_stack(StackValue::Integer(i64::from(*value))),
            Instruction::SIPush(value) =>
                stack_frame.push_stack(StackValue::Integer(i64::from(*value))),
            Instruction::ILoad(offset) => Runtime::exec_iload(&mut stack_frame, usize::from(*offset))?,
            Instruction::ILoad0(()) => Runtime::exec_iload(&mut stack_frame, 0)?,
            Instruction::ILoad1(()) => Runtime::exec_iload(&mut stack_frame, 1)?,
            Instruction::ILoad2(()) => Runtime::exec_iload(&mut stack_frame, 2)?,
            Instruction::ILoad3(()) => Runtime::exec_iload(&mut stack_frame, 3)?,
            // 20..
            // 30..
            Instruction::IStore(offset) => Runtime::exec_istore(&mut stack_frame, usize::from(*offset))?,
            Instruction::IStore0(()) => Runtime::exec_istore(&mut stack_frame, 0)?,

            Instruction::IStore1(()) => Runtime::exec_istore(&mut stack_frame, 1)?,

            Instruction::IStore2(()) => Runtime::exec_istore(&mut stack_frame, 2)?,

            Instruction::IStore3(()) => Runtime::exec_istore(&mut stack_frame, 3)?,
            // 40..
            // 50..
            // 60..
            Instruction::IAdd(()) => match (stack_frame.pop_stack(), stack_frame.pop_stack()) {
                (Some(StackValue::Integer(lh)), Some(StackValue::Integer(rh))) =>
                    stack_frame.push_stack(StackValue::Integer(lh + rh)),
                (Some(_), Some(_)) =>
                    return Err(RuntimeError::StackType { expected: format!("integer") }),
                (None, None) | (Some(_), None) =>
                    return Err(RuntimeError::EmptyStack),
                _ =>
                    return Err(RuntimeError::GenericError { message: format!("IAdd") })
            }
            // 80..
            Instruction::IInc((offset, value)) => match stack_frame.get_variable_mut(usize::from(*offset)) {
                Some(LocalVariable::Integer(intvalue)) => *intvalue = *intvalue + 1,
                Some(_) => return Err(RuntimeError::VariableType { expected: format!("integer"), offset: usize::from(*offset) }),
                None => return Err(RuntimeError::VariableOutOfScope)
            }

            // a0..
            Instruction::IfICmpGE(instruction) => {
                // would be nice to know the instruction offset now…
                // additionally we need a way to "jump" to that instruction. currently we are just looping from top to bottom
            }

            Instruction::Goto(offset) => {
                /*let new_offset = offset as i64 + instruction_counter as i64;
                println!("GOTO: {} + {} = {}", offset, instruction_counter, new_offset);
                if new_offset < 0 {
                    panic!();
                }
                ins.goto(usize::from(new_offset as u16));*/
            }

            Instruction::IReturn(()) => return match stack_frame.pop_stack() {
                Some(StackValue::Integer(ret)) => Ok(InstructionResult::Return(Some(StackValue::Integer(ret)))),
                Some(_) => Err(RuntimeError::StackType { expected: format!("Integer") }),
                None => Err(RuntimeError::EmptyStack)
            },

            // b0..
            Instruction::Return(()) => return Ok(InstructionResult::Return(None)),
            Instruction::InvokeStatic(method_offset) => {
                let class = &context.class;
                match class.get_constant(*method_offset) {
                    Some(ConstantType::MethodRef { class_index, name_and_type_index }) => {
                        let cls_name = {
                            let other_class = self.class_index_map.get(class.get_class_name()).unwrap().get(&(*class_index as usize));
                            if other_class.is_none() {
                                return Err(RuntimeError::GenericError { message: format!("class not found {}", class_index) });
                            }
                            other_class.unwrap().clone()
                        };


                        if cls_name.eq(class.get_class_name()) {
                            let method = match class.get_method_from_nat(*name_and_type_index) {
                                Some(m) => m,
                                None => return Err(RuntimeError::MethodNotFound)
                            };

                            let mut args = method.get_signature().arguments.iter().map(|arg_type| {
                                //TODO: we really should check the type here. some day.
                                match stack_frame.pop_stack() {
                                    Some(StackValue::Integer(intvalue)) => Ok(LocalVariable::Integer(intvalue)),
                                    Some(StackValue::None) => Ok(LocalVariable::None), //??? None => undefined, Null => null.
                                    Some(StackValue::Null) => Ok(LocalVariable::Null),
                                    None => Err(RuntimeError::EmptyStack)
                                }
                            }).collect::<Result<Vec<LocalVariable>, RuntimeError>>()?;
                            args.reverse();

                            println!("{:?}, {:?}", method, args);
                            let return_val = self.run_method(method, class.clone(), args);
                            println!(" => return value: {:?}", return_val);
                            match return_val {
                                Ok(Some(stack_value)) => stack_frame.push_stack(stack_value),
                                Ok(None) => (),
                                Err(err) => return Err(err)
                            };
                        }
                        //
                    }
                    Some(_) => {
                        return Err(RuntimeError::GenericError {
                            message: format!("invalid method offset {}", method_offset)
                        });
                    }
                    None => {
                        return Err(RuntimeError::GenericError {
                            message: format!("invalid method offset {}", method_offset)
                        });
                    }
                }
            }
            _ => return Err(RuntimeError::GenericError { message: format!("unknown instruction") })
        };

        Ok(InstructionResult::Continue)
    }

    fn run_method(&mut self, method: &Method, class: Arc<ClassFile<'a>>, arguments: Vec<LocalVariable>) -> Result<Option<StackValue>, RuntimeError> {
        println!("running method {}", method.name);
        let mut stack_frame = StackFrame::for_method(method, arguments);
        let mut return_value: Option<StackValue> = None;
        println!("{:?}", stack_frame);
        let mut instruction_counter: usize = 0;
        let mut ins = method.instructions();
        let mut context = Context {
            return_value: None,
            class: class.clone(),
        };

        while let Some(instruction) = ins.next() {
            println!("{}: {:?}, {}", instruction_counter, instruction, instruction.get_size());

            match self.exec(&instruction, &mut stack_frame, &mut context) {
                Ok(InstructionResult::Continue) => { /* nop, just keep executing */ }
                Ok(InstructionResult::Goto(offset)) => {
                    //TODO
                }
                Ok(InstructionResult::Return(return_value)) => {
                    self.check_return_type(method.get_signature().return_type, &return_value);
                    return Ok(return_value);
                }
                Err(err) => return Err(err)
            }

            instruction_counter += instruction.get_size();
            println!("{:?}, return {:?}", stack_frame, context.return_value);
        }

        Err(RuntimeError::GenericError { message: format!("reached end of method with no return") })
    }

    /// this is just here for internal verification.
    /// the compiler should prevent these type of errors.
    /// if something like this happens, the jvm has f**ked up, or the bytecode is broken
    fn check_return_type(&self, return_type: ValueType, return_value: &Option<StackValue>) -> Result<(), RuntimeError> {
        return match return_type {
            ValueType::Void => if return_value.is_some() {
                Err(RuntimeError::GenericError { message: format!("invalid return type. expected void.") })
            } else {
                Ok(())
            },
            ValueType::Integer => match return_value {
                Some(StackValue::Integer(_)) => Ok(()),
                Some(StackValue::Null) => Ok(()),
                _ => Err(RuntimeError::GenericError { message: format!("invalid return type. expected integer.") })
            },
            _ => Ok(()),
        };
    }
}

#[cfg(test)]
mod test {
    use java::class_file::ClassFile;
    use java::class_file::Method;
    use java::class_file::read_class_file;
    use java::runtime::Runtime;
    use java::runtime::StackValue;

    #[test] //TODO
    fn test_basic_math() {
        let simple_match_sample = include_bytes!("../../../sample/SimpleMath.class");
        let class = read_class_file(simple_match_sample).unwrap().1;
        let mut rt = Runtime::create(class);
        let result = rt.exec_method_on_main("testMe").unwrap();

        assert_eq!(Some(StackValue::Integer(46)), result)
    }
}
