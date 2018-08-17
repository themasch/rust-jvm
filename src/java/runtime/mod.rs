use java::class_file::Method;
use std::collections::HashMap;
use java::class_file::ClassFile;
use std::path::PathBuf;
use std::sync::Arc;
use java::class_file::ConstantType;

#[derive(Debug, Fail)]
pub enum RuntimeError {
    #[fail(display = "runtime error: {}", message)]
    GenericError { message: String },
    #[fail(display = "runtime error: top of stack had the wrong type. expected: {}", expected)]
    StackType { expected: String },
    #[fail(display = "runtime error: stack poped when empty")]
    EmptyStack,
    #[fail(display = "runtime error: stack poped when empty")]
    MethodNotFound,
}

#[derive(Debug)]
enum LocalVariable {
    None,
    Integer(i64),
}

#[derive(Debug)]
enum StackValue {
    None,
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

    //TODO: Result benutzten, du esel!
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
    //TODO: Result benutzten, du esel!
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

    fn run_method(&mut self, method: &Method, class: Arc<ClassFile<'a>>, arguments: Vec<LocalVariable>) -> Result<Option<StackValue>, RuntimeError> {
        println!("running method {}", method.name);
        use java::instructions::Instruction;
        let mut stack_frame = StackFrame::for_method(method, arguments);
        let mut return_value: Option<StackValue> = None;
        println!("{:?}", stack_frame);
        for instruction in method.instructions() {
            println!("{:?}", instruction);
            match instruction {
                // 10: pushes a byte onto the stack as an integer
                Instruction::BIPush(value) =>
                    stack_frame.push_stack(StackValue::Integer(i64::from(value))),
                Instruction::SIPush(value) =>
                    stack_frame.push_stack(StackValue::Integer(i64::from(value))),

                Instruction::IConstm1(()) => stack_frame.push_stack(StackValue::Integer(-1)),
                Instruction::IConst0(()) => stack_frame.push_stack(StackValue::Integer(0)),
                Instruction::IConst1(()) => stack_frame.push_stack(StackValue::Integer(1)),
                Instruction::IConst2(()) => stack_frame.push_stack(StackValue::Integer(2)),
                Instruction::IConst3(()) => stack_frame.push_stack(StackValue::Integer(3)),
                Instruction::IConst4(()) => stack_frame.push_stack(StackValue::Integer(4)),
                Instruction::IConst5(()) => stack_frame.push_stack(StackValue::Integer(5)),

                Instruction::IStore(offset) => {
                    // 36: store int value into variable 1
                    Runtime::exec_istore(&mut stack_frame, usize::from(offset))?;
                }
                Instruction::IStore0(()) => {
                    // 3b: store int value into variable 1
                    Runtime::exec_istore(&mut stack_frame, 0)?;
                }
                Instruction::IStore1(()) => {
                    // 3c: store int value into variable 1
                    Runtime::exec_istore(&mut stack_frame, 1)?;
                }
                Instruction::IStore2(()) => {
                    // 3d: store int value into variable 1
                    Runtime::exec_istore(&mut stack_frame, 2)?;
                }
                Instruction::IStore3(()) => {
                    // 3e: store int value into variable 3
                    Runtime::exec_istore(&mut stack_frame, 3)?;
                }

                Instruction::ILoad(offset) => {
                    //15: load an int from local variable {offset} to the stack
                    Runtime::exec_iload(&mut stack_frame, usize::from(offset))?;
                }
                Instruction::ILoad0(()) => {
                    //1a: load an int from local variable 0 to the stack
                    Runtime::exec_iload(&mut stack_frame, 0)?;
                }
                Instruction::ILoad1(()) => {
                    //1a: load an int from local variable 1 to the stack
                    Runtime::exec_iload(&mut stack_frame, 1)?;
                }
                Instruction::ILoad2(()) => {
                    //1c: load an int from local variable 2 to the stack
                    Runtime::exec_iload(&mut stack_frame, 2)?;
                }
                Instruction::ILoad3(()) => {
                    //1d: load an int from local variable 3 to the stack
                    Runtime::exec_iload(&mut stack_frame, 3)?;
                }

                Instruction::IAdd(()) => {
                    match (stack_frame.pop_stack(), stack_frame.pop_stack()) {
                        (Some(StackValue::Integer(lh)), Some(StackValue::Integer(rh))) => {
                            stack_frame.push_stack(StackValue::Integer(lh + rh));
                        }
                        _ => return Err(RuntimeError::GenericError { message: format!("IAdd") })
                    }
                }

                Instruction::IfICmpGE(instruction) => {
                    // would be nice to know the instruction offset now…
                }

                Instruction::IReturn(()) => {
                    match stack_frame.pop_stack() {
                        Some(StackValue::Integer(ret)) => return_value = Some(StackValue::Integer(ret)),
                        Some(_) => return Err(RuntimeError::StackType { expected: format!("Integer") }),
                        None => return Err(RuntimeError::EmptyStack)
                    }
                }

                Instruction::Return(()) => {
                    return_value = None;
                }

                // b8: invoke a static method
                Instruction::InvokeStatic(method_offset) => {
                    match class.get_constant(method_offset) {
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
                                        None => Err(RuntimeError::EmptyStack)
                                    }
                                }).collect::<Result<Vec<LocalVariable>, RuntimeError>>()?;
                                args.reverse();

                                println!("{:?}, {:?}", method, args);
                                match self.run_method(method, class.clone(), args) {
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
            }

            println!("{:?}, return {:?}", stack_frame, return_value);
        }

        Ok(return_value)
    }
}
