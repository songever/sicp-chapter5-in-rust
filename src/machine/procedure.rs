
use super::Machine;

pub trait Executor: CloneExecutor {
    type Oprands;

    fn execute(&self, machine: &mut Machine, oprands: Self::Oprands) -> Vec<u32>;
}

pub trait CloneExecutor {
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

pub type Operation = Box<dyn Executor<Oprands = Vec<u32>>>;

impl Clone for Operation {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub type Procedure = Box<dyn Fn(&mut Machine)>;
pub type ValueProcedure = Box<dyn Fn(&mut Machine) -> Vec<u32>>;

pub fn combine_procedures(procedures: Vec<ValueProcedure>) -> Result<ValueProcedure, String> {
    Ok(Box::new(move |machine: &mut Machine| {
        procedures.iter().flat_map(|proc| proc(machine)).collect()
    }))
}
