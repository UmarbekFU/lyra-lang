/// Tracks local variables during compilation.
#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: usize,
}

/// Tracks upvalues (captured variables) during compilation.
#[derive(Debug, Clone)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

/// Scope state for the compiler.
#[derive(Debug)]
pub struct ScopeTracker {
    pub locals: Vec<Local>,
    pub upvalues: Vec<Upvalue>,
    pub scope_depth: usize,
}

impl ScopeTracker {
    pub fn new() -> Self {
        ScopeTracker {
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) -> usize {
        let mut pop_count = 0;
        while let Some(local) = self.locals.last() {
            if local.depth < self.scope_depth {
                break;
            }
            self.locals.pop();
            pop_count += 1;
        }
        self.scope_depth -= 1;
        pop_count
    }

    pub fn add_local(&mut self, name: String) -> usize {
        let index = self.locals.len();
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
        index
    }

    pub fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        // Check if we already have this upvalue
        for (i, uv) in self.upvalues.iter().enumerate() {
            if uv.index == index && uv.is_local == is_local {
                return i;
            }
        }
        let idx = self.upvalues.len();
        self.upvalues.push(Upvalue { index, is_local });
        idx
    }
}
