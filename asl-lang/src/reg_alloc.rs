use ast::{Children, Source};
use function_signatures::FunctionSignatureRegisters;
use name_resolution::Vars;
use parity_wasm::elements::ValueType;
use specs::prelude::*;
use std::collections::HashSet;
use types::Ty;

#[derive(Component)]
pub struct Register(pub u32);

#[derive(Component, Default)]
pub struct FunctionRegisters {
    pub i32s: u32,
    pub i64s: u32,
    pub f32s: u32,
    pub f64s: u32,
}

#[derive(Default)]
struct RegisterLists {
    i32s: HashSet<Entity>,
    i64s: HashSet<Entity>,
    f32s: HashSet<Entity>,
    f64s: HashSet<Entity>,
}

pub struct RegAlloc<'s>(pub &'s Source);

impl<'a, 's> System<'a> for RegAlloc<'s> {
    type SystemData = (
        ReadStorage<'a, Vars>,
        ReadStorage<'a, Children>,
        ReadStorage<'a, Ty>,
        WriteStorage<'a, FunctionRegisters>,
        WriteStorage<'a, Register>,
        ReadStorage<'a, FunctionSignatureRegisters>,
    );

    fn run(
        &mut self,
        (vars, children, types, mut function_registers, mut register_storage, function_signatures): Self::SystemData,
){
        let mut registers = RegisterLists::default();

        for (_, fn_entity) in self.0.code_items() {
            let mut id = if let Some(params) = function_signatures.get(fn_entity) {
                params.0.len()
            } else {
                0
            };

            reg_alloc(
                &mut registers,
                &register_storage,
                &vars,
                &children,
                &types,
                fn_entity,
            );

            let _ = function_registers.insert(
                fn_entity,
                FunctionRegisters {
                    i32s: registers.i32s.len() as u32,
                    i64s: registers.i64s.len() as u32,
                    f32s: registers.f32s.len() as u32,
                    f64s: registers.f64s.len() as u32,
                },
            );

            for var in registers
                .i32s
                .drain()
                .chain(registers.i64s.drain())
                .chain(registers.f32s.drain())
                .chain(registers.f64s.drain())
            {
                let _ = register_storage.insert(var, Register(id as u32));
                id += 1;
            }
        }
    }
}

fn reg_alloc(
    registers: &mut RegisterLists,
    register_storage: &WriteStorage<Register>,
    vars: &ReadStorage<Vars>,
    children: &ReadStorage<Children>,
    types: &ReadStorage<Ty>,
    entity: Entity,
) {
    if let Some(vars) = vars.get(entity) {
        for var in &vars.0 {
            let ty = types.get(*var).unwrap();
            if let (Some(reg_ty), None) = (ty.reg_type(), register_storage.get(*var)) {
                match reg_ty {
                    ValueType::I32 => registers.i32s.insert(*var),
                    ValueType::I64 => registers.i64s.insert(*var),
                    ValueType::F32 => registers.f32s.insert(*var),
                    ValueType::F64 => registers.f64s.insert(*var),
                };
            }
        }
    }

    if let Some(my_children) = children.get(entity) {
        for child in &my_children.0 {
            reg_alloc(registers, register_storage, vars, children, types, *child);
        }
    }
}

impl Ty {
    pub fn reg_type(&self) -> Option<ValueType> {
        match self {
            Ty::Bool | Ty::U8 | Ty::U16 | Ty::U32 | Ty::I8 | Ty::I16 | Ty::I32 => {
                Some(ValueType::I32)
            }
            Ty::I64 | Ty::U64 => Some(ValueType::I64),
            Ty::F32 => Some(ValueType::F32),
            Ty::F64 => Some(ValueType::F64),
            Ty::Unit => None,
            _ => unreachable!("This general type shouldn't make it to the reg alloc"),
        }
    }
}
