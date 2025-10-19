
mod parser;
mod procedure;
use parser::{Expr, ControllerText, Instruction};
use parser::{PrimitiveExpr, ValueExpr};
use parser::parse;

use std::vec;
use std::{cell::RefCell, collections::HashMap};
use std::rc::Rc;

use crate::machine::parser::OpreationExpr;

#[derive(Debug)]
struct Register {
    contents: RefCell<Option<u32>>,
}
impl Register {
    fn make_register() -> Self {
        Register { contents: RefCell::new(None) }
    }
    fn get_content(&self) -> u32 {
        self.contents.borrow().unwrap()
    }
    fn set_content(&self, value: u32) {
        self.contents.replace(Some(value));
    }
}
impl Clone for Register {
    fn clone(&self) -> Self {
        Register { contents: self.contents.clone() }
    }
}

trait Executor: CloneExecutor {
    type Oprands;

    fn execute(&self, machine: &mut Machine, oprands: Self::Oprands) -> Vec<u32>;
}

trait CloneExecutor {
    fn clone_box(&self) -> Box<dyn Executor<Oprands = Vec<u32>>>;
}
impl<T> CloneExecutor for T
where
    T: 'static + Executor<Oprands = Vec<u32>> + Clone,
{
    fn clone_box(&self) -> Box<dyn Executor<Oprands = Vec<u32>>> {
        Box::new(self.clone())
    }
}

type Operation = Box<dyn Executor<Oprands = Vec<u32>>>;
impl Clone for Operation {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
type Procedure = Box<dyn Fn(&mut Machine)>;
type ValueProcedure = Box<dyn Fn(&mut Machine) -> Vec<u32>>;
type Bool = u32;
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
    the_operations: HashMap<String, Operation>,
    stack: Stack,
    the_instruction_sequence: Vec<Procedure>,
}
impl Machine {
    pub fn make_machine(register_names: &[&str], ops: Vec<(String, Operation)>, controller_text: &str) -> Result<Self, String> {
        let mut machine = Machine::default();
        machine.register_table.insert("pc".to_string(), Register::make_register());
        machine.register_table.insert("flag".to_string(), Register::make_register());
        for name in register_names {
            machine.allocate_register(name);
        }
        machine.install_operations(ops);

        parse(controller_text)
            .map_err(|e| format!("Parsing controller text error: {e}"))
            .and_then(move |(_, text)|{
                let instructions = machine.assemble(text)?;
                machine.install_instruction_sequence(instructions);
                Ok(machine)
            })
    }
    
// assemble is used before install_sequences in make_machine, so assemble should be added to impl Machine
// How to implement instruction-to-behavior matching using enum match?
//    Assembly should generate enums!
// In the original text, Procedure is the process to be executed, constructing Instruction as a pair (text, proc),
// When execute is called, it directly runs (cdr Instruction), so assembly must be called before running to generate procs in insts
// During assembly, instructions are written via (set-cdr! inst)
    fn assemble(&mut self, controller_text: ControllerText) -> Result<Vec<Procedure>, String> {
        let mut insts = Vec::new();
        let mut label_table = HashMap::new();

        self.extract_labels(controller_text, &mut insts, &mut label_table);
        
        let mut procedures = Vec::new();
        for inst in insts {
            let proc = self.make_exec_proc(inst, &label_table)?;
            procedures.push(proc);
        }
    
        Ok(procedures)
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
    fn make_exec_proc(&mut self, instruction: Instruction, labels: &HashMap<String, usize>) -> Result<Procedure, String> {
        
        match instruction {
            Instruction::Assign { target_reg, val_expr } => {
                let target = self.get_register(&target_reg)?.clone();
                let exec_val_expr = self.make_val_expr_exec(&val_expr, labels)?;
                Ok(Box::new(move |machine: &mut Machine| {
                    let value = exec_val_expr(machine)[0];
                    target.set_content(value);
                }))
            }
            Instruction::Branch(label) => {
                let label_name = label.get_name();
                let target_pc = *labels
                    .get(&label_name)
                    .ok_or_else(|| format!("Label '{}' not found", label_name))?;
                Ok(Box::new(move |machine: &mut Machine| {
                    machine.set_pc(target_pc as u32);
                }))
            }
            Instruction::Test(cond) => {
                let condition = self.make_operation_exec(&cond, labels)?;
                Ok(Box::new(move |machine: &mut Machine| {
                    let new_flag = condition(machine)[0];
                    machine.set_flag(new_flag);
                }))
            }
        }

    }

    //We have to make sure the operation error to be handled while assembling 
    fn make_val_expr_exec(&mut self, expr: &ValueExpr, labels: &HashMap<String, usize>) -> Result<ValueProcedure, String> {
        match expr {
            ValueExpr::OpreationExpr(op) => {
                let proc = self.make_operation_exec(&op, labels)?;
                    Ok(Box::new(move |machine: &mut Machine| {
                        proc(machine)
                    }))
                }
                ValueExpr::PrimitiveExpr(PrimitiveExpr::Constant(value)) => {
                    let value = *value;
                    Ok(Box::new(move |_machine: &mut Machine| vec![value]))
                }
                ValueExpr::PrimitiveExpr(PrimitiveExpr::Label(label)) => {
                    let label_name = label.get_name();
                    labels
                        .get(&label_name)
                        .cloned()
                        .map(|index| {
                            Box::new(move |_machine: &mut Machine| vec![index as u32]) as ValueProcedure
                        })
                        .ok_or_else(|| format!("Label '{}' not found", label_name))
                }
                ValueExpr::PrimitiveExpr(PrimitiveExpr::Register(reg)) => {
                    let reg = self.get_register(reg)?;
                    let contents = reg.get_content();
                    Ok(Box::new(move |_machine: &mut Machine| vec![contents]))
                }
        }
    }

    fn make_operation_exec(&mut self, op: &OpreationExpr, labels: &HashMap<String, usize>) -> Result<ValueProcedure, String> {
        if op.oprands().len() != op.arity() {
            return Err(format!(
                "Operation '{}' expects {} operands, but got {}",
                op.name(),
                op.arity(),
                op.oprands().len()
            ));
        }

        // 1. 将所有 oprands 转换为 Procedure
        let procedures: Result<Vec<ValueProcedure>, String> = op
            .oprands()
            .into_iter()
            .map(|val_expr| self.make_val_expr_exec(val_expr, labels))
            .collect();
        let procedures = procedures?;

        // 2. 合并所有 Procedure 为一个 Procedure
        let oprands_proc = combine_procedures(procedures)?;

        // 3. 获取操作
        let operation = self.get_operation(op.name())?.clone();
        
        // 4. 生成最终的闭包
        Ok(Box::new(move |machine: &mut Machine| {
            let oprands = oprands_proc(machine);
            operation.execute(machine, oprands)
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
    fn install_operations(&mut self, ops: Vec<(String, Operation)>) {
        for (op_name, op) in ops {
            self.the_operations.insert(op_name.to_string(), op);
        }
    }
    pub fn get_register(&mut self, name: &str) -> Result<&Register, String> {
        self.register_table.get(name).ok_or(format!("Unknown register: {name}"))
    }
    pub fn get_operation(&self, name: &str) -> Result<Operation, String> {
        self.the_operations.get(name).cloned().ok_or(format!("Unknown operation: {name}"))    
    }
    pub fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }
    pub fn operations(&mut self) -> &mut HashMap<String, Operation> {
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
    fn set_pc(&mut self, new_pc: u32) {
        self.register_table.entry("pc".to_string())
            .and_modify(|pc| pc.set_content(new_pc));
    }
    fn set_flag(&mut self, new_flag: Bool) {
        self.register_table.entry("flag".to_string())
            .and_modify(|flag| flag.set_content(new_flag));
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
