mod parser;
use parser::Expr;
use parser::Instruction;

use std::{cell::RefCell, collections::HashMap};
use std::rc::Rc;

#[derive(Default)]
// Error type isn't assigned yet.
// The register_table includes two special registers 'pc' and 'flag'.
struct Machine {
    register_table: HashMap<String, Register>,
    the_operations: HashMap<String, Opreation>,
    stack: Stack,
    the_instruction_sequence: Vec<Instruction>,
}
impl Machine {
    pub fn make_machine(register_names: &[&str], ops: Vec<(String, Opreation)>, controller_text: &str) -> Self {
        let mut machine = Machine::default();
        machine.register_table.insert("pc".to_string(), Register::make_register());
        machine.register_table.insert("flag".to_string(), Register::make_register());
        for name in register_names {
            machine.allocate_register(name);
        }
        machine.install_operations(ops);
        machine.install_instruction_sequence(machine.assemble(controller_text));
        machine
    }
    
// assemble is used before install_sequences in make_machine, so assemble should be added to impl Machine
// How to implement instruction-to-behavior matching using enum match?
//    Assembly should generate enums!
// In the original text, Procedure is the process to be executed, constructing Instruction as a pair (text, proc),
// When execute is called, it directly runs (cdr Instruction), so assembly must be called before running to generate procs in insts
// During assembly, instructions are written via (set-cdr! inst)
    fn assemble(&mut self, controller_text: &str) -> Vec<Instruction> {
        let label_table = HashMap::new();
        let mut insts = Vec::new();

        let make_label_entry = |label_name, insts_index|
        let lookup_label = |name| label_table.get(name)?;
        
    }

// extract_labels recursively parses text, expanding lambda(insts, labels) layer by layer, where the innermost lambda is update_insts,
// The parameters (insts, labels) in update_insts are continuously applied to lambda and cons-connected into a complete list according to the definition
// Key concept search: "continuation" procedure
// This technique can be replaced with Rust iteration
    fn extract_labels(text: &[], receive)
        {extract_labels(cdr text) receive}
        
// update_insts: iterate through instructions
    fn update_insts(&mut self, insts, labels) 
        {}
}
impl Machine {
    fn install_instruction_sequence(&mut self, seq: &[Instruction]) {}
    fn allocate_register(&mut self, name: &str) {
        self.register_table.insert(name.to_string(), Register::make_register());
    }
    fn install_operations(&mut self, ops: Vec<(String, Opreation)>) {
        for (op_name, op) in ops {
            self.the_operations.insert(op_name.to_string(), op);
        }
    }
    
    pub fn get_register(&self, name: &str) -> Option<u32> {
        self.register_table.get(name).map(|register| register.get_content())
    }
    pub fn stack(&self) -> &Stack {
        &self.stack
    }
    pub fn operations(&self) -> &HashMap<String, Opreation> {
        &self.the_operations
    }
    
    pub fn start(&self) {}
}

impl Machine {
    fn advance_pc(&mut self) {
        self.register_table.entry("pc".to_string())
            .and_modify(|pc| {
                pc.contents.borrow_mut().map(|val| val + 1);
            });
    }
}

type Opreation = Box<dyn Fn(&Machine)>;

#[derive(Debug)]
struct Register {
    contents: RefCell<Option<u32>>,
}
impl Register {
    fn make_register() -> Self {
        Register { contents: RefCell::new(None) }
    }
    fn get_content(& self) -> u32 {
        self.contents.borrow().unwrap()
    }
    fn set_content(&mut self) {
        self.contents.replace(Some(0));
    }
}

#[derive(Debug)]
struct Stack(Rc<RefCell<Vec<u32>>>);
impl Default for Stack {
    fn default() -> Self {
        Stack::make_stack()
    }
}
impl Stack {
    fn make_stack() -> Self {
        Stack(Rc::new(RefCell::new(Vec::new())))
    }
    fn push(&self, value: u32) {
        self.0.borrow_mut().push(value);
    }
    fn pop(&self) -> Option<u32> {
        self.0.borrow_mut().pop()
    }
    fn initialize(&self) {
        self.0.borrow_mut().clear();
    }
}
