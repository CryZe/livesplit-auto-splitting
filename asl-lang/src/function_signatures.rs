use debug_info::SrcByteRange;
use error::{RangeError, RangeResult, ResultExt};
use name_resolution::FunctionDecl;
use name_resolution::Vars;
use parity_wasm::elements::ValueType;
use reg_alloc::Register;
use specs::prelude::*;
use types::Ty;

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct FunctionSignatureRegisters(pub Vec<ValueType>);

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct FunctionCall {
    pub arguments: usize,
}

pub struct AllocFunctionSignatureRegisters;

impl<'a> System<'a> for AllocFunctionSignatureRegisters {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FunctionDecl>,
        ReadStorage<'a, Ty>,
        WriteStorage<'a, FunctionSignatureRegisters>,
        WriteStorage<'a, Register>,
        ReadStorage<'a, Vars>,
    );

    fn run(
        &mut self,
        (entities, decls, types, mut signatures, mut registers, vars): Self::SystemData,
    ) {
        for (entity, FunctionDecl(_, params)) in (&*entities, &decls).join() {
            let mut signature = Vec::new();
            for param in params {
                if let Some(reg_ty) = types.get(*param).and_then(|t| t.reg_type()) {
                    // Ugly assumption that it's always the first one
                    let var = vars.get(*param).unwrap().0[0];
                    registers
                        .insert(var, Register(signature.len() as u32))
                        .unwrap();
                    signature.push(reg_ty);
                }
            }
            signatures
                .insert(entity, FunctionSignatureRegisters(signature))
                .unwrap();
        }
    }
}

pub struct VerifyFunctionCallSignatures {
    result: RangeResult<()>,
}

impl VerifyFunctionCallSignatures {
    pub fn new() -> Self {
        Self { result: Ok(()) }
    }
    pub fn run(mut self, world: &World) -> RangeResult<()> {
        self.run_now(&world.res);
        self.result
    }
}

impl<'a> System<'a> for VerifyFunctionCallSignatures {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, SrcByteRange>,
        ReadStorage<'a, FunctionCall>,
        ReadStorage<'a, FunctionDecl>,
        ReadStorage<'a, Vars>,
    );

    fn run(&mut self, (entities, ranges, function_calls, function_decls, vars): Self::SystemData) {
        for (entity, function_call, Vars(vars)) in (&*entities, &function_calls, &vars).join() {
            let resolved_function = vars[0];
            let FunctionDecl(_, params) = function_decls.get(resolved_function).unwrap();
            let expected = params.len();
            let has = function_call.arguments;
            if expected != has {
                self.result = Err(RangeError::new(format!(
                    "Provided {} arguments but the function expects {} arguments.",
                    has, expected
                ))).with_entity_range(entity, &ranges);
                return;
            }
        }
    }
}
