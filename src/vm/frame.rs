use crate::compiler::bytecode::FunctionProto;
use crate::eval::value::Value;

/// A single call frame on the VM's call stack.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The function being executed.
    pub function: FunctionProto,
    /// Instruction pointer (index into function's chunk.code).
    pub ip: usize,
    /// Base index into the VM's value stack for this frame's locals.
    pub stack_base: usize,
    /// Captured upvalues for closures.
    pub upvalues: Vec<Value>,
}

impl CallFrame {
    pub fn new(function: FunctionProto, stack_base: usize, upvalues: Vec<Value>) -> Self {
        CallFrame {
            function,
            ip: 0,
            stack_base,
            upvalues,
        }
    }
}
