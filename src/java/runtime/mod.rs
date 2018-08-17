use java::class_file::Method;
use std::collections::HashMap;
use java::class_file::ClassFile;
use std::path::PathBuf;
use std::sync::Arc;
use java::class_file::ConstantType;

#[derive(Debug)]
enum LocalVariable {}

#[derive(Debug)]
enum StackValue {}

#[derive(Debug)]
struct StackFrame {
    local_variables: Vec<LocalVariable>,
    stack: Vec<StackValue>,
}

impl StackFrame {
    fn create(var_count: usize, stack_size: usize) -> StackFrame {
        StackFrame {
            local_variables: Vec::with_capacity(var_count),
            stack: Vec::with_capacity(stack_size),
        }
    }

    fn for_method(method: &Method) -> StackFrame {
        let locals = usize::from(method.get_code().unwrap().max_locals);
        let stack = usize::from(method.get_code().unwrap().max_stack);

        StackFrame::create(locals, stack)
    }

    fn get_variable(&mut self, index: usize) -> Option<&mut LocalVariable> {
        self.local_variables.get_mut(index)
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

        self.run_method(method.unwrap(), class.clone());
    }

    fn run_method(&mut self, method: &Method, class: Arc<ClassFile<'a>>) {
        println!("running method {}", method.name);
        use java::instructions::Instruction;
        let mut stack_frame = StackFrame::for_method(method);
        for instruction in method.instructions() {
            println!("{:?}", instruction);
            match instruction {
                Instruction::InvokeStatic(method_offset) => {
                    match class.get_constant(method_offset) {
                        Some(ConstantType::MethodRef { class_index, name_and_type_index }) => {
                            let cls_name = {
                                let other_class = self.class_index_map.get(class.get_class_name()).unwrap().get(&(*class_index as usize));
                                if other_class.is_none() {
                                    eprintln!("class not found {}", class_index);
                                    return;
                                }

                                other_class.unwrap().clone()
                            };

                            if cls_name.eq(class.get_class_name()) {
                                //let method = class.methods.get(nam)
                                //self.run_method(method, class.clone());
                            }
                            //
                        }
                        Some(_) => {
                            eprintln!("invalid method offset {}", method_offset);
                            return;
                        }
                        None => {
                            eprintln!("method offset not valid {}", method_offset);
                            return;
                        }
                    }
                }
                _ => ()
            }
        }
    }
}
