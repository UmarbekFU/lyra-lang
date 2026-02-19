use crate::eval::value::Value;
use crate::span::Span;

/// A single bytecode instruction.
#[derive(Debug, Clone)]
pub enum Op {
    /// Push a constant from the constant pool.
    Constant(usize),
    /// Push unit value.
    Unit,
    /// Push true.
    True,
    /// Push false.
    False,
    /// Discard top of stack.
    Pop,

    // ── Variable access ──
    /// Push local variable at stack offset.
    GetLocal(usize),
    /// Set local variable at stack offset.
    SetLocal(usize),
    /// Push captured upvalue.
    GetUpvalue(usize),
    /// Push global variable by name.
    GetGlobal(String),
    /// Define a global variable.
    DefineGlobal(String),

    // ── Arithmetic ──
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Negate,

    // ── Comparison ──
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // ── Logic ──
    Not,

    // ── Control flow ──
    /// Jump forward by offset.
    Jump(usize),
    /// Jump forward if top of stack is false (pops condition).
    JumpIfFalse(usize),
    /// Loop backward by offset.
    Loop(usize),

    // ── Functions ──
    /// Call function with N arguments.
    Call(u8),
    /// Tail call: reuse current frame.
    TailCall(u8),
    /// Return from function.
    Return,
    /// Create a closure from function prototype at constant index.
    /// Second field: number of upvalues to capture.
    Closure(usize, Vec<UpvalueRef>),

    // ── Data structures ──
    /// Create a list from N values on the stack.
    MakeList(usize),
    /// Create a tuple from N values.
    MakeTuple(usize),
    /// Cons: push head :: tail.
    Cons,
    /// Create an ADT value with constructor name and N fields.
    MakeAdt(String, usize),

    // ── Pattern matching ──
    /// Test if top of stack is an ADT with given tag. Jump to offset if not.
    TestTag(String, usize),
    /// Test if top of stack equals the int literal. Jump if not.
    TestInt(i64, usize),
    /// Test if top of stack equals the bool. Jump if not.
    TestBool(bool, usize),
    /// Test if top of stack equals the string. Jump if not.
    TestString(String, usize),
    /// Test if top of stack is unit. Jump if not.
    TestUnit(usize),
    /// Test if list is empty. Jump if not.
    TestEmptyList(usize),
    /// Test if list is non-empty (for cons patterns). Jump if empty.
    TestCons(usize),
    /// Test if tuple has N elements. Jump if not.
    TestTuple(usize, usize),
    /// Duplicate top of stack.
    Dup,
    /// Get field at index from ADT on top of stack.
    GetAdtField(usize),
    /// Get head of list.
    GetListHead,
    /// Get tail of list.
    GetListTail,
    /// Get tuple element at index.
    GetTupleField(usize),
    /// Pop and discard (for failed pattern cleanup).
    PopMatch,

    // ── Records (Phase 3) ──
    /// Create record from N key-value pairs.
    MakeRecord(Vec<String>),
    /// Get field from record.
    GetField(String),

    // ── String interpolation (Phase 2) ──
    /// Convert top of stack to string.
    ToString,
    /// Concatenate two strings.
    StringConcat,

    /// Swap top two stack values.
    Swap,
    /// Keep TOS, pop N values underneath it.
    PopUnder(usize),

    /// Print top of stack with newline.
    Print,
    /// Print top of stack without newline.
    PrintRaw,
}

/// Reference to a captured variable for closures.
#[derive(Debug, Clone)]
pub struct UpvalueRef {
    /// If true, capture from the enclosing function's locals.
    /// If false, capture from the enclosing function's upvalues.
    pub is_local: bool,
    /// Index into the locals or upvalues array.
    pub index: usize,
}

/// A compiled function prototype.
#[derive(Debug, Clone)]
pub struct FunctionProto {
    pub name: String,
    pub arity: u8,
    pub chunk: Chunk,
    pub upvalue_count: usize,
}

/// A chunk of bytecode with its constant pool.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Op>,
    pub constants: Vec<Value>,
    pub spans: Vec<Span>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
        }
    }

    pub fn emit(&mut self, op: Op, span: Span) -> usize {
        let idx = self.code.len();
        self.code.push(op);
        self.spans.push(span);
        idx
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.code.len() - offset - 1;
        match &mut self.code[offset] {
            Op::Jump(ref mut target)
            | Op::JumpIfFalse(ref mut target)
            | Op::TestTag(_, ref mut target)
            | Op::TestInt(_, ref mut target)
            | Op::TestBool(_, ref mut target)
            | Op::TestString(_, ref mut target)
            | Op::TestUnit(ref mut target)
            | Op::TestEmptyList(ref mut target)
            | Op::TestCons(ref mut target)
            | Op::TestTuple(_, ref mut target) => {
                *target = jump;
            }
            _ => panic!("Not a jump instruction at offset {}", offset),
        }
    }

    pub fn current_offset(&self) -> usize {
        self.code.len()
    }

    /// Disassemble for debugging.
    pub fn disassemble(&self, name: &str) -> String {
        let mut out = format!("== {} ==\n", name);
        for (i, op) in self.code.iter().enumerate() {
            out.push_str(&format!("{:04} {:?}\n", i, op));
        }
        out
    }
}
