use ast::{Children, Source};
use function_signatures::FunctionSignatureRegisters;
use name_resolution::Vars;
use parity_wasm::elements::ValueType;
use specs::prelude::*;
use types::{Tuple, Ty};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Registers(pub Vec<Register>);

type Register = Option<(ValueType, u32)>;

#[derive(Component, Default, Clone)]
#[storage(DenseVecStorage)]
pub struct FunctionRegisters {
    pub i32s: u32,
    pub i64s: u32,
    pub f32s: u32,
    pub f64s: u32,
}

pub struct RegAlloc<'s>(pub &'s Source);

impl<'a, 's> System<'a> for RegAlloc<'s> {
    type SystemData = (
        ReadStorage<'a, Vars>,
        ReadStorage<'a, Children>,
        ReadStorage<'a, Ty>,
        WriteStorage<'a, FunctionRegisters>,
        WriteStorage<'a, Registers>,
        ReadStorage<'a, FunctionSignatureRegisters>,
    );

    fn run(
        &mut self,
        (
            vars,
            children,
            types,
            mut function_registers_storage,
            mut registers_storage,
            function_signatures,
        ): Self::SystemData,
    ) {
        let mut entities_that_need_slots = Vec::new();

        for (_, fn_entity) in self.0.code_items() {
            let first_local_id = if let Some(params) = function_signatures.get(fn_entity) {
                params.0.len()
            } else {
                0
            };

            let mut function_registers = FunctionRegisters::default();

            reg_alloc(
                &mut function_registers,
                &mut entities_that_need_slots,
                &mut registers_storage,
                &vars,
                &children,
                &types,
                fn_entity,
            );

            let mut reg_indices = function_registers.clone();
            reg_indices.i32s = first_local_id as u32;
            reg_indices.i64s = reg_indices.i32s + function_registers.i32s;
            reg_indices.f32s = reg_indices.i64s + function_registers.i64s;
            reg_indices.f64s = reg_indices.f32s + function_registers.f32s;

            let _ = function_registers_storage.insert(fn_entity, function_registers);

            for entity in entities_that_need_slots.drain(..) {
                for reg in &mut registers_storage.get_mut(entity).unwrap().0 {
                    if let Some((reg_ty, idx)) = reg {
                        match reg_ty {
                            ValueType::I32 => {
                                *idx = reg_indices.i32s;
                                reg_indices.i32s += 1;
                            }
                            ValueType::I64 => {
                                *idx = reg_indices.i64s;
                                reg_indices.i64s += 1;
                            }
                            ValueType::F32 => {
                                *idx = reg_indices.f32s;
                                reg_indices.f32s += 1;
                            }
                            ValueType::F64 => {
                                *idx = reg_indices.f64s;
                                reg_indices.f64s += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn reg_alloc(
    function_registers: &mut FunctionRegisters,
    entities_that_need_slots: &mut Vec<Entity>,
    registers_storage: &mut WriteStorage<Registers>,
    vars: &ReadStorage<Vars>,
    children: &ReadStorage<Children>,
    types: &ReadStorage<Ty>,
    entity: Entity,
) {
    // TODO Limit this to actual declarations.
    if let Some(vars) = vars.get(entity) {
        for var in &vars.0 {
            if registers_storage.get(*var).is_none() {
                let ty = types.get(*var).unwrap();
                let ty_registers = ty.create_registers_description();
                for (reg_ty, _) in ty_registers.0.iter().flatten() {
                    match reg_ty {
                        ValueType::I32 => function_registers.i32s += 1,
                        ValueType::I64 => function_registers.i64s += 1,
                        ValueType::F32 => function_registers.f32s += 1,
                        ValueType::F64 => function_registers.f64s += 1,
                    }
                }
                let _ = registers_storage.insert(*var, ty_registers);
                entities_that_need_slots.push(*var);
            }
        }
    }

    if let Some(my_children) = children.get(entity) {
        for child in &my_children.0 {
            reg_alloc(
                function_registers,
                entities_that_need_slots,
                registers_storage,
                vars,
                children,
                types,
                *child,
            );
        }
    }
}

impl Ty {
    fn populate_registers(&self, registers: &mut Vec<Register>) {
        let trivial_ty = match self {
            Ty::Bool | Ty::U8 | Ty::U16 | Ty::U32 | Ty::I8 | Ty::I16 | Ty::I32 => {
                Some(ValueType::I32)
            }
            Ty::I64 | Ty::U64 => Some(ValueType::I64),
            Ty::F32 => Some(ValueType::F32),
            Ty::F64 => Some(ValueType::F64),
            Ty::Unit => None,
            Ty::Tuple(Tuple(types)) => {
                for ty in types.read().unwrap().iter() {
                    ty.as_ref().unwrap().populate_registers(registers);
                }
                return;
            }
            _ => unreachable!("This general type shouldn't make it to the reg alloc"),
        };
        registers.push(trivial_ty.map(|t| (t, 0)));
    }
    pub fn create_registers_description(&self) -> Registers {
        let mut registers = Vec::new();
        self.populate_registers(&mut registers);
        Registers(registers)
    }
    pub fn value_type(&self) -> Option<ValueType> {
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
