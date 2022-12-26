use std::collections::HashMap;
use std::iter::Enumerate;
use std::slice::Iter;

use boolvec::BoolVec;
use dash_middle::compiler::instruction::Instruction;
use llvm_sys::core::LLVMDoubleType;
use llvm_sys::core::LLVMInt1Type;
use llvm_sys::core::LLVMInt64Type;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;

#[derive(Debug, Clone)]
pub enum Type {
    I64,
    F64,
    Boolean,
}

impl Type {
    pub fn to_llvm_type(&self) -> LLVMTypeRef {
        unsafe {
            match self {
                Self::I64 => LLVMInt64Type(),
                Self::F64 => LLVMDoubleType(),
                Self::Boolean => LLVMInt1Type(),
            }
        }
    }
}

pub trait InferQueryProvider {
    fn type_of_local(&self, index: u16) -> Option<Type>;
    fn type_of_constant(&self, index: u16) -> Option<Type>;
    fn did_take_nth_branch(&self, nth: usize) -> bool;
}

struct DecodeContext<'a> {
    iter: Enumerate<Iter<'a, u8>>,
    type_stack: Vec<Type>,
    local_types: HashMap<u16, Type>,
    /// Instruction pointer maps to whether this is the start of a label
    labels: BoolVec,
}

impl<'a> DecodeContext<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            iter: bytes.iter().enumerate(),
            type_stack: Vec::new(),
            local_types: HashMap::new(),
            labels: BoolVec::filled_with(bytes.len(), false),
        }
    }

    /// Pops the last two values off the stack.
    pub fn pop_two(&mut self) -> (Type, Type) {
        let a = self.type_stack.pop().unwrap();
        let b = self.type_stack.pop().unwrap();
        (b, a)
    }

    pub fn push(&mut self, ty: Type) {
        self.type_stack.push(ty);
    }

    pub fn pop(&mut self) -> Option<Type> {
        self.type_stack.pop()
    }

    pub fn next_instruction(&mut self) -> Option<(usize, Instruction)> {
        self.iter.next().map(|(a, &b)| (a, Instruction::from_repr(b).unwrap()))
    }

    pub fn next_byte(&mut self) -> u8 {
        self.iter.next().map(|(_, &b)| b).unwrap()
    }

    pub fn next_wide(&mut self) -> u16 {
        let a = self.next_byte();
        let b = self.next_byte();
        u16::from_ne_bytes([a, b])
    }

    pub fn next_wide_signed(&mut self) -> i16 {
        self.next_wide() as i16
    }

    pub fn set_inferred_type(&mut self, index: u16, ty: Type) {
        self.local_types.insert(index, ty);
    }

    pub fn set_label_at(&mut self, at: usize) {
        self.labels.set(at, true);
    }
}

#[derive(Debug)]
pub enum InferError {
    UnsupportedTypeBiInstruction {
        instr: Instruction,
        left: Type,
        right: Type,
    },
    UnsupportedTypeInstruction {
        instr: Instruction,
    },
}

pub struct InferResult {
    pub local_tys: HashMap<u16, Type>,
    pub labels: BoolVec,
}

pub fn infer_types_and_labels<Q: InferQueryProvider>(bytecode: &[u8], query: Q) -> Result<InferResult, InferError> {
    let mut iter = bytecode.iter();
    let mut cx = DecodeContext::new(bytecode);
    let mut branch_count = 0;

    while let Some((index, instr)) = cx.next_instruction() {
        match instr {
            Instruction::Add | Instruction::Sub | Instruction::Mul | Instruction::Div | Instruction::Rem => {
                let (left, right) = cx.pop_two();

                match (&left, &right) {
                    (Type::I64, Type::I64) => cx.push(Type::I64),
                    (Type::F64, Type::F64) => cx.push(Type::F64),
                    (Type::I64 | Type::F64, Type::I64 | Type::F64) => cx.push(Type::F64),
                    (Type::Boolean, Type::Boolean) => cx.push(Type::Boolean),
                    _ => return Err(InferError::UnsupportedTypeBiInstruction { instr, left, right }),
                }
            }
            Instruction::Constant | Instruction::ConstantW => {
                let index = match instr {
                    Instruction::Constant => cx.next_byte().into(),
                    Instruction::ConstantW => cx.next_wide(),
                    _ => unreachable!(),
                };

                let ty = query.type_of_constant(index);
                match ty {
                    Some(ty) => cx.push(ty),
                    None => return Err(InferError::UnsupportedTypeInstruction { instr }),
                }
            }
            Instruction::LdLocal | Instruction::LdLocalW => {
                let index = match instr {
                    Instruction::LdLocal => cx.next_byte().into(),
                    Instruction::LdLocalW => cx.next_wide(),
                    _ => unreachable!(),
                };

                let ty = query.type_of_local(index);
                match ty {
                    Some(ty) => cx.push(ty),
                    None => return Err(InferError::UnsupportedTypeInstruction { instr }),
                }
            }
            Instruction::StoreLocal | Instruction::StoreLocalW => {
                let index = match instr {
                    Instruction::StoreLocal => cx.next_byte().into(),
                    Instruction::StoreLocalW => cx.next_wide(),
                    _ => unreachable!(),
                };

                let ty = query.type_of_local(index);
                match ty {
                    Some(ty) => {
                        let value = cx.pop().unwrap();
                        cx.set_inferred_type(index, ty);
                    }
                    None => return Err(InferError::UnsupportedTypeInstruction { instr }),
                }
            }
            Instruction::Lt
            | Instruction::Le
            | Instruction::Gt
            | Instruction::Ge
            | Instruction::Eq
            | Instruction::Ne
            | Instruction::StrictEq
            | Instruction::StrictNe => {
                let _ = cx.pop_two();
                cx.push(Type::Boolean);
            }
            Instruction::Jmp => {
                let n = cx.next_wide_signed() + 3;
                let target_ip = index as i16 + n;
                for _ in 0..n {
                    cx.next_byte();
                }
                cx.set_label_at(target_ip as usize);
            }
            Instruction::JmpFalseP | Instruction::JmpNullishP | Instruction::JmpTrueP | Instruction::JmpUndefinedP => {
                let _ = cx.pop();
                let count = cx.next_wide_signed() + 3;
                let target_ip = index as i16 + count;

                if query.did_take_nth_branch(branch_count) {
                    for _ in 0..count {
                        cx.next_byte();
                    }

                    cx.set_label_at(target_ip as usize);
                }

                branch_count += 1;
            }
            Instruction::Pop => drop(cx.pop()),
            _ => return Err(InferError::UnsupportedTypeInstruction { instr }),
        }
    }

    Ok(InferResult {
        labels: cx.labels,
        local_tys: cx.local_types,
    })
}
