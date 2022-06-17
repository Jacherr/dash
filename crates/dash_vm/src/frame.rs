use std::collections::BTreeMap;
use std::rc::Rc;

use dash_middle::compiler::constant::Constant;
use dash_middle::compiler::CompileResult;

use crate::gc::handle::Handle;
use crate::gc::trace::Trace;

use super::value::function::user::UserFunction;
use super::value::object::Object;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct TryBlock {
    pub catch_ip: usize,
    pub frame_ip: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Exports {
    pub default: Option<Value>,
    pub named: Vec<(Rc<str>, Value)>,
}

#[derive(Debug, Clone)]
pub enum FrameState {
    /// Regular function
    Function {
        /// Whether the currently executing function is a constructor call
        is_constructor_call: bool,
    },
    /// Top level frame of a module
    Module(Exports),
}

#[derive(Debug, Clone, Default)]
pub struct LoopCounter(u32);

impl LoopCounter {
    pub fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn is_hot(&self) -> bool {
        self.0 > 5
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub name: Option<Rc<str>>,
    pub ip: usize,
    pub reserved_stack_size: usize,
    pub constants: Rc<[Constant]>,
    pub externals: Rc<[Handle<dyn Object>]>,
    pub this: Option<Value>,
    pub buffer: Rc<[u8]>,
    pub sp: usize,
    pub state: FrameState,

    /// Counts the number of backjumps to a particular loop header, to find hot loops
    pub loop_counter: BTreeMap<usize, LoopCounter>
}

unsafe impl Trace for Frame {
    fn trace(&self) {
        self.externals.trace();
    }
}

impl Frame {
    pub fn from_function(
        name: Option<Rc<str>>,
        this: Option<Value>,
        uf: &UserFunction,
        is_constructor_call: bool,
    ) -> Self {
        Self {
            name,
            this,
            buffer: uf.buffer().clone(),
            constants: uf.constants().clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            reserved_stack_size: uf.locals(),
            state: FrameState::Function { is_constructor_call },
            loop_counter: BTreeMap::new(),
        }
    }

    pub fn from_module(name: Option<Rc<str>>, this: Option<Value>, uf: &UserFunction) -> Self {
        Self {
            name,
            this,
            buffer: uf.buffer().clone(),
            constants: uf.constants().clone(),
            externals: uf.externals().clone(),
            ip: 0,
            sp: 0,
            reserved_stack_size: uf.locals(),
            state: FrameState::Module(Exports::default()),
            loop_counter: BTreeMap::new(),
        }
    }

    pub fn is_module(&self) -> bool {
        matches!(self.state, FrameState::Module(_))
    }

    pub fn from_compile_result(cr: CompileResult) -> Self {
        // it's [logically] impossible to create a Frame if the compile result references external values
        // there's likely a bug somewhere if this assertion fails and will be *really* confusing if this invariant doesn't get caught
        debug_assert!(cr.externals.is_empty());

        Self {
            name: None,
            this: None,
            buffer: cr.instructions.into(),
            constants: cr.cp.into_vec().into(),
            externals: Vec::new().into(),
            ip: 0,
            sp: 0,
            reserved_stack_size: cr.locals,
            state: FrameState::Function {
                is_constructor_call: false,
            },
            loop_counter: BTreeMap::new(),
        }
    }

    pub fn set_reserved_stack_size(&mut self, size: usize) {
        self.reserved_stack_size = size;
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }
}
