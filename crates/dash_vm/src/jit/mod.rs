use std::rc::Rc;

mod frontend;
mod query;
use dash_log::debug;
use dash_log::error;
use dash_log::warn;
pub use frontend::Frontend;
use frontend::Trace;

use crate::Vm;

fn handle_loop_trace(vm: &mut Vm, jmp_instr_ip: usize) {
    let (trace, fun) = match frontend::compile_current_trace(vm) {
        Ok(v) => v,
        Err(err) => {
            error!("JIT compilation failed! {err:?}");
            vm.poison_ip(jmp_instr_ip);
            return;
        }
    };

    let frame_sp = {
        let frame = vm.frames.last().unwrap();
        frame.sp
    };

    let offset_ip = trace.start();
    let mut target_ip = 0;
    unsafe {
        let stack_ptr = vm.stack.as_mut_ptr().cast();
        let frame_sp = u64::try_from(frame_sp).unwrap();
        fun(stack_ptr, frame_sp, &mut target_ip);
    }

    target_ip += offset_ip as u64;
    debug!("jit returned");
    debug!(target_ip);

    // `target_ip` is not the "real" IP, there may be some extra instructions before the loop header
    vm.frames.last_mut().unwrap().ip = target_ip as usize;
}

pub fn handle_loop_end(vm: &mut Vm, loop_end_ip: usize) {
    let frame = vm.frames.last_mut().unwrap();

    // We are jumping back to a loop header
    if let Some(trace) = vm.jit.recording_trace() {
        // There is a trace being recorded for this loop
        if frame.ip == trace.start() {
            handle_loop_trace(vm, loop_end_ip);
        } else {
            todo!("Side exit");
        }
    } else {
        handle_loop_counter_inc(vm, loop_end_ip, None);
    }
}

fn handle_loop_counter_inc(vm: &mut Vm, loop_end_ip: usize, parent_ip: Option<usize>) {
    let frame = vm.frames.last_mut().unwrap();
    let origin = Rc::as_ptr(&frame.function);
    let counter = frame.loop_counter.get_or_insert(frame.ip);

    counter.inc();
    if counter.is_hot() {
        if frame.function.is_poisoned_ip(loop_end_ip) {
            // We have already tried to compile this loop, and failed
            // So don't bother re-tracing
            warn!("loop is poisoned, cannot jit");
            return;
        }

        // Hot loop detected
        // Start recording a trace (i.e. every opcode) for the next loop iteration
        // The trace will go on until either:
        // - The loop is exited
        // - The iteration has ended (i.e. we are here again)
        debug!("detected hot loop, begin trace");
        debug!(loop_end_ip, parent_ip);
        let trace = Trace::new(origin, frame.ip, loop_end_ip);
        vm.jit.set_recording_trace(trace);
    }
}

#[cfg(all(test, feature = "jit"))]
mod tests {

    use dash_compiler::FunctionCompiler;
    use dash_llvm_jit_backend::codegen;
    use dash_llvm_jit_backend::codegen::CodegenQuery;
    use dash_llvm_jit_backend::codegen::JitConstant;
    use dash_llvm_jit_backend::passes::bb_generation::BBGenerationQuery;
    use dash_llvm_jit_backend::passes::bb_generation::ConditionalBranchAction;
    use dash_llvm_jit_backend::passes::type_infer::Type;
    use dash_llvm_jit_backend::passes::type_infer::TypeInferQuery;
    use dash_llvm_jit_backend::typed_cfg;
    use dash_llvm_jit_backend::typed_cfg::TypedCfgQuery;
    use dash_optimizer::OptLevel;

    use crate::value::primitive::Number;
    use crate::value::Value;

    #[derive(Debug)]
    struct TestQueryProvider {}
    impl BBGenerationQuery for TestQueryProvider {
        fn conditional_branch_at(&self, ip: usize) -> ConditionalBranchAction {
            match ip {
                0xB => ConditionalBranchAction::NotTaken,
                _ => todo!(),
            }
        }
    }

    impl TypeInferQuery for TestQueryProvider {
        fn type_of_constant(&self, index: u16) -> dash_llvm_jit_backend::passes::type_infer::Type {
            match index {
                0 | 1 | 2 => Type::I64,
                _ => todo!("{index}"),
            }
        }

        fn type_of_local(&self, index: u16) -> Type {
            match index {
                0 => Type::I64,
                1 => Type::Boolean,
                o => todo!("{o}"),
            }
        }
    }

    impl CodegenQuery for TestQueryProvider {
        fn get_constant(&self, cid: u16) -> JitConstant {
            match cid {
                0 => JitConstant::I64(0),
                1 => JitConstant::I64(10),
                2 => JitConstant::I64(3),
                _ => todo!(),
            }
        }
    }

    impl TypedCfgQuery for TestQueryProvider {}

    #[test]
    pub fn llvm() {
        let cr = FunctionCompiler::compile_str(
            r"

        for (let i = 0; i < 10; i++) {
            let x = i > 3;
        }
        ",
            OptLevel::None,
        )
        .unwrap();
        let bytecode = &cr.instructions;
        let mut query = TestQueryProvider {};
        let tcfg = typed_cfg::lower(bytecode, &mut query).unwrap();
        dbg!(&tcfg);

        dash_llvm_jit_backend::init();

        let fun = codegen::compile_typed_cfg(bytecode, &tcfg, &mut query).unwrap();
        let mut s = [Value::Number(Number(0.0)), Value::Boolean(false)];
        let mut x = 0;
        unsafe { fun(s.as_mut_ptr().cast(), 0, &mut x) };
        dbg!(x, s);
    }
}
