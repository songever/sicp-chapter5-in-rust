mod parser;
use parser::{Expr, ControllerText, Instruction};
use parser::{PrimitiveExpr, ValueExpr};
use parser::parse;

use std::{cell::RefCell, collections::HashMap};
use std::rc::Rc;

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
type Opreation = Box<dyn Fn(&mut Machine, Vec<Oprand>)>;
type Oprand = u32;

type Procedure = Box<dyn Fn(&mut Machine)>;
type ValueProcedure = Box<dyn Fn(&mut Machine) -> Vec<u32>>;
fn combine_procedures(procedures: Vec<ValueProcedure>) -> Result<ValueProcedure, String> {
    Ok(Box::new(move |machine: &mut Machine| {
        procedures
            .iter()
            .flat_map(|proc| proc(machine))
            .collect()
    }))
}

#[derive(Default)]
// Error type isn't assigned yet.
// The register_table includes two special registers 'pc' and 'flag'.
struct Machine {
    register_table: HashMap<String, Register>,
    the_operations: HashMap<String, Opreation>,
    stack: Stack,
    the_instruction_sequence: Vec<Procedure>,
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
        machine.install_instruction_sequence(machine.assemble(parse(controller_text)));
        machine
    }
    
// assemble is used before install_sequences in make_machine, so assemble should be added to impl Machine
// How to implement instruction-to-behavior matching using enum match?
//    Assembly should generate enums!
// In the original text, Procedure is the process to be executed, constructing Instruction as a pair (text, proc),
// When execute is called, it directly runs (cdr Instruction), so assembly must be called before running to generate procs in insts
// During assembly, instructions are written via (set-cdr! inst)
    fn assemble(&mut self, controller_text: ControllerText) -> Vec<Procedure> {
        let mut insts = Vec::new();
        let mut label_table = HashMap::new();
        let lookup_label = |name: &str| label_table.get(name);

        self.extract_labels(controller_text, &mut insts, &mut label_table);
        
        
    }


// extract_labels recursively parses text, expanding lambda(insts, labels) layer by layer, where the innermost lambda is update_insts,
// The parameters (insts, labels) in update_insts are continuously applied to lambda and cons-connected into a complete list according to the definition
// Key concept search: "continuation" procedure
//     Here this technique will be replaced simply by Rust iteration
    fn extract_labels(&mut self, text: ControllerText, insts: &mut Vec<Instruction>, labels: &mut HashMap<String, usize>) {
        text.into_iter().for_each(|expr| {
            match expr {
                Expr::Instruction(instruction) => {
                    insts.push(instruction);
                }
                Expr::Label(label) => {
                    labels.insert(label.get_name(), insts.len());
                }
            }
        });  
    }
        
// update_insts: iterate through instructions
    fn update_insts(&mut self, instruction:&mut Instruction, labels: &HashMap<String, usize>) -> Result<Procedure, String> {
        
        match instruction {
            Instruction::Assign { target_reg, val_expr } => {
                let target = self.get_register(target_reg)?;
                let exec_val_expr = match val_expr {
                    ValueExpr::OpreationExpr(op) => {
                        let opreation = self.get_operation(op.name())?
                        
                    }
                    ValueExpr::PrimitiveExpr(PrimitiveExpr::Constant()) => {
                    }
                    ValueExpr::
                };
            }
            Instruction::Branch(label) => {

            }
            Instruction::Test(cond) => {

            }
        }

    }

    fn make_val_expr_exec(&mut self, expr: ValueExpr) -> Result<ValueProcedure, String> {
        match expr {
            ValueExpr::OpreationExpr(op) => {
                    let operation = self.get_operation(op.name())?
                    op
                }
                ValueExpr::PrimitiveExpr(PrimitiveExpr::Constant(value)) => {
                }
                ValueExpr::PrimitiveExpr(PrimitiveExpr::Label(label)) => {

                }
                ValueExpr::PrimitiveExpr(PrimitiveExpr::Register(reg)) => {

                }
        }
    }
    fn make_operation_exec(&mut self, operation: Opreation, oprands: Vec<ValueExpr>) -> Result<Procedure, String> {
        // 1. 将所有 oprands 转换为 Procedure
        let procedures: Result<Vec<ValueProcedure>, String> = oprands
            .into_iter()
            .map(|val_expr| self.make_val_expr_exec(val_expr))
            .collect();
        let procedures = procedures?;

        // 2. 合并所有 Procedure 为一个 Procedure
        let oprands_proc = combine_procedures(procedures)?;

        // 3. 生成最终的闭包：先调用 oprands_proc 生成参数，再执行 operation
        Ok(Box::new(move |machine: &mut Machine| {
            // 调用 oprands_proc 生成参数列表
            let oprands = oprands_proc(machine);
            // 用 oprands 执行 operation（示例：假设 operation 是一个函数）
            operation(machine, oprands)
        }))
    }
}
impl Machine {
    fn install_instruction_sequence(&mut self, seq: Vec<Procedure>) {
        self.the_instruction_sequence = seq;
    }
    fn allocate_register(&mut self, name: &str) {
        self.register_table.insert(name.to_string(), Register::make_register());
    }
    fn install_operations(&mut self, ops: Vec<(String, Opreation)>) {
        for (op_name, op) in ops {
            self.the_operations.insert(op_name.to_string(), op);
        }
    }
    pub fn get_register_mut(&mut self, name: &str) -> Result<&mut Register, String> {
        self.register_table.get_mut(name).ok_or(format!("Unknown register: {name}"))
    }
    pub fn get_operation(&self, name: &str) -> Result<&Opreation, String> {
        self.the_operations.get(name).ok_or(format!("Unknown operation: {name}"))    
    }
    pub fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }
    pub fn operations(&mut self) -> &mut HashMap<String, Opreation> {
        &mut self.the_operations
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
