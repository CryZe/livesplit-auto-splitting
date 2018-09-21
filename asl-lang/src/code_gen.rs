use ast::{ActionKind, Source, State};
use function_indexing::FunctionIndex;
use function_signatures::FunctionSignatureRegisters;
use name_resolution::Vars;
use parity_wasm::{
    builder::{ImportBuilder, ModuleBuilder, SignatureBuilder},
    elements::{BlockType, Instruction, Instructions, Local, Module, ValueType},
};
use reg_alloc::{FunctionRegisters, Register};
use reg_extend::NeedsExtending;
use specs::prelude::*;
use types::Ty;

pub enum Op {
    Entity(Entity),
    Add,
    Sub,
    Mul,
    Div,
    LShift,
    RShift,
    Not,
    Neg,
    BoolOr,
    BoolAnd,
    BitOr,
    BitAnd,
    Xor,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    If,
    Else,
    End,
    Loop,
    Block,
    Extend(Entity),
    ExtendVar(usize),
    Cast(Entity),
    Br(u32),
    BrIf(u32),
    ConstInt(i64),
    ConstFloat(f64),
    ConstBool(bool),
    Drop,
    LoadVar(usize),
    StoreVar(usize),
    StateVar(bool, String),
    Call(usize),
}

#[derive(Component)]
pub struct CodeGenDesc(pub Vec<Op>);

pub struct CodeGen<'s>(pub &'s Source, pub Option<Module>);

impl<'a, 's> System<'a> for CodeGen<'s> {
    type SystemData = (
        ReadStorage<'a, CodeGenDesc>,
        ReadStorage<'a, Ty>,
        ReadStorage<'a, Vars>,
        ReadStorage<'a, Register>,
        ReadStorage<'a, FunctionRegisters>,
        ReadStorage<'a, NeedsExtending>,
        ReadStorage<'a, FunctionIndex>,
        ReadStorage<'a, FunctionSignatureRegisters>,
    );

    fn run(
        &mut self,
        (
            codegen_descs,
            types,
            vars,
            registers,
            function_registers,
            needs_extending,
            function_indices,
            function_signatures,
        ): Self::SystemData,
    ) {
        let state = self.0.state().unwrap();

        macro_rules! sig {
            ($ty:ident) => {
                SignatureBuilder::new()
                    .params()
                    .i32()
                    .i32()
                    .build()
                    .return_type()
                    .$ty()
                    .build_sig()
            };
        };

        let sigs = vec![
            SignatureBuilder::new()
                .params()
                .i32()
                .i32()
                .build()
                .build_sig(),
            SignatureBuilder::new()
                .params()
                .i32()
                .i32()
                .i32()
                .build()
                .return_type()
                .i32()
                .build_sig(),
            SignatureBuilder::new()
                .params()
                .i32()
                .i64()
                .build()
                .build_sig(),
            sig!(i32),
            sig!(i64),
            sig!(f32),
            sig!(f64),
        ];

        let mut builder = ModuleBuilder::new()
            .memory()
            .with_min(1)
            .build()
            .export()
            .field("memory")
            .internal()
            .memory(0)
            .build()
            .with_signatures(sigs)
            .import()
            .module("env")
            .field("set_process_name")
            .external()
            .func(0)
            .build()
            .import()
            .module("env")
            .field("push_pointer_path")
            .external()
            .func(1)
            .build()
            .import()
            .module("env")
            .field("push_offset")
            .external()
            .func(2)
            .build()
            .data()
            .offset(Instruction::I32Const(0))
            .value(state.process.as_bytes().to_vec())
            .build();

        macro_rules! import {
            ($name:expr, $idx:expr) => {
                builder.push_import(
                    ImportBuilder::new()
                        .module("env")
                        .field($name)
                        .external()
                        .func($idx)
                        .build(),
                );
            };
        };

        import!("get_u8", 3);
        import!("get_u16", 3);
        import!("get_u32", 3);
        import!("get_u64", 4);
        import!("get_i8", 3);
        import!("get_i16", 3);
        import!("get_i32", 3);
        import!("get_i64", 4);
        import!("get_f32", 5);
        import!("get_f64", 6);

        let mut builder = builder
            .export()
            .field("configure")
            .internal()
            .func(13)
            .build();

        let mut configure_fn = vec![
            Instruction::I32Const(0),
            Instruction::I32Const(state.process.len() as i32),
            Instruction::Call(0),
        ];

        let mut data_section_offset = state.process.len();
        for (id, path) in state.paths.iter().enumerate() {
            configure_fn.push(Instruction::I32Const(data_section_offset as i32));
            configure_fn.push(Instruction::I32Const(path.module.len() as i32));
            configure_fn.push(Instruction::I32Const(0)); // TODO Type
            configure_fn.push(Instruction::Call(1));
            configure_fn.push(Instruction::Drop);

            for offset in &path.offsets {
                configure_fn.push(Instruction::I32Const(id as i32));
                configure_fn.push(Instruction::I64Const(*offset));
                configure_fn.push(Instruction::Call(2));
            }

            builder = builder
                .data()
                .offset(Instruction::I32Const(data_section_offset as i32))
                .value(path.module.as_bytes().to_vec())
                .build();
            data_section_offset += path.module.len();
        }

        configure_fn.push(Instruction::End);

        builder = builder
            .function()
            .signature()
            .build()
            .body()
            .with_instructions(Instructions::new(configure_fn))
            .build()
            .build();

        for (fn_kind, fn_entity) in self.0.code_items() {
            let FunctionIndex(fn_index) = function_indices.get(fn_entity).unwrap();
            builder = build_action(
                *fn_index,
                fn_kind,
                state,
                builder,
                &codegen_descs,
                &types,
                &vars,
                &registers,
                &function_registers,
                &needs_extending,
                &function_indices,
                &function_signatures,
                fn_entity,
            );
        }

        self.1 = Some(builder.build());
    }
}

fn build_action(
    fn_idx: u32,
    fn_kind: Option<ActionKind>,
    state: &State,
    builder: ModuleBuilder,
    codegen_descs: &ReadStorage<CodeGenDesc>,
    types: &ReadStorage<Ty>,
    vars: &ReadStorage<Vars>,
    registers: &ReadStorage<Register>,
    function_registers: &ReadStorage<FunctionRegisters>,
    needs_extending: &ReadStorage<NeedsExtending>,
    function_indices: &ReadStorage<FunctionIndex>,
    function_signatures: &ReadStorage<FunctionSignatureRegisters>,
    entity: Entity,
) -> ModuleBuilder {
    let (ret_type, name, params) = match fn_kind {
        Some(ActionKind::Start) => (Some(ValueType::I32), Some("should_start"), Vec::new()),
        Some(ActionKind::Split) => (Some(ValueType::I32), Some("should_split"), Vec::new()),
        Some(ActionKind::Reset) => (Some(ValueType::I32), Some("should_reset"), Vec::new()),
        Some(ActionKind::IsLoading) => (Some(ValueType::I32), Some("is_loading"), Vec::new()),
        Some(ActionKind::GameTime) => (Some(ValueType::F64), Some("game_time"), Vec::new()),
        None => (
            types.get(entity).unwrap().reg_type(),
            None,
            function_signatures.get(entity).unwrap().0.clone(),
        ),
    };

    let mut instructions = Vec::new();
    code_gen(
        &mut instructions,
        state,
        codegen_descs,
        types,
        vars,
        registers,
        needs_extending,
        function_indices,
        entity,
    );
    instructions.push(Instruction::End);

    let mut builder = builder
        .function()
        .signature()
        .with_params(params)
        .with_return_type(ret_type)
        .build()
        .body()
        .with_locals(build_locals(entity, function_registers))
        .with_instructions(Instructions::new(instructions))
        .build()
        .build();

    if let Some(name) = name {
        builder = builder.export().field(name).internal().func(fn_idx).build();
    }

    builder
}

fn build_locals(entity: Entity, function_registers: &ReadStorage<FunctionRegisters>) -> Vec<Local> {
    let regs = function_registers.get(entity).unwrap();
    let mut locals = Vec::new();
    if regs.i32s > 0 {
        locals.push(Local::new(regs.i32s, ValueType::I32));
    }
    if regs.i64s > 0 {
        locals.push(Local::new(regs.i64s, ValueType::I64));
    }
    if regs.f32s > 0 {
        locals.push(Local::new(regs.f32s, ValueType::F32));
    }
    if regs.f64s > 0 {
        locals.push(Local::new(regs.f64s, ValueType::F64));
    }
    locals
}

fn code_gen(
    instructions: &mut Vec<Instruction>,
    state: &State,
    codegen_descs: &ReadStorage<CodeGenDesc>,
    types: &ReadStorage<Ty>,
    vars: &ReadStorage<Vars>,
    registers: &ReadStorage<Register>,
    needs_extending: &ReadStorage<NeedsExtending>,
    function_indices: &ReadStorage<FunctionIndex>,
    entity: Entity,
) {
    let desc = codegen_descs.get(entity).unwrap();
    let ty = types.get(entity).unwrap();
    let reg_ty = ty.reg_type();
    for op in &desc.0 {
        match op {
            Op::Entity(child) => code_gen(
                instructions,
                state,
                codegen_descs,
                types,
                vars,
                registers,
                needs_extending,
                function_indices,
                *child,
            ),
            Op::Add => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Add,
                    ValueType::I64 => Instruction::I64Add,
                    ValueType::F32 => Instruction::F32Add,
                    ValueType::F64 => Instruction::F64Add,
                };
                instructions.push(ins);
            }
            Op::Sub => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Sub,
                    ValueType::I64 => Instruction::I64Sub,
                    ValueType::F32 => Instruction::F32Sub,
                    ValueType::F64 => Instruction::F64Sub,
                };
                instructions.push(ins);
            }
            Op::Mul => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Mul,
                    ValueType::I64 => Instruction::I64Mul,
                    ValueType::F32 => Instruction::F32Mul,
                    ValueType::F64 => Instruction::F64Mul,
                };
                instructions.push(ins);
            }
            Op::Div => {
                let ins = match (reg_ty.unwrap(), ty.is_uint()) {
                    (ValueType::I32, true) => Instruction::I32DivU,
                    (ValueType::I32, false) => Instruction::I32DivS,
                    (ValueType::I64, true) => Instruction::I64DivU,
                    (ValueType::I64, false) => Instruction::I64DivS,
                    (ValueType::F32, _) => Instruction::F32Div,
                    (ValueType::F64, _) => Instruction::F64Div,
                };
                instructions.push(ins);
            }
            Op::LShift => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Shl,
                    ValueType::I64 => Instruction::I64Shl,
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::RShift => {
                let ins = match (reg_ty.unwrap(), ty.is_uint()) {
                    (ValueType::I32, true) => Instruction::I32ShrU,
                    (ValueType::I32, false) => Instruction::I32ShrS,
                    (ValueType::I64, true) => Instruction::I64ShrU,
                    (ValueType::I64, false) => Instruction::I64ShrS,
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::Not => {
                let ins = match (ty, reg_ty.unwrap()) {
                    (Ty::Bool, _) => Instruction::I32Eqz,
                    (_, ValueType::I32) => {
                        instructions.push(Instruction::I32Const(!0));
                        Instruction::I32Xor
                    }
                    (_, ValueType::I64) => {
                        instructions.push(Instruction::I64Const(!0));
                        Instruction::I64Xor
                    }
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::Neg => {
                // Negating should work just fine with dirty values.
                let ins = match reg_ty.expect("Can't negate unit values") {
                    ValueType::I32 => {
                        instructions.push(Instruction::I32Const(0));
                        Instruction::I32Sub
                    }
                    ValueType::I64 => {
                        instructions.push(Instruction::I64Const(0));
                        Instruction::I64Sub
                    }
                    ValueType::F32 => Instruction::F32Neg,
                    ValueType::F64 => Instruction::F64Neg,
                };
                instructions.push(ins);
            }
            Op::BoolOr => instructions.push(Instruction::I32Or),
            Op::BoolAnd => instructions.push(Instruction::I32And),
            Op::BitOr => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Or,
                    ValueType::I64 => Instruction::I64Or,
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::BitAnd => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32And,
                    ValueType::I64 => Instruction::I64And,
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::Xor => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Xor,
                    ValueType::I64 => Instruction::I64Xor,
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::Eq => {
                let ins = match reg_ty {
                    Some(ValueType::I32) => Instruction::I32Eq,
                    Some(ValueType::I64) => Instruction::I64Eq,
                    Some(ValueType::F32) => Instruction::F32Eq,
                    Some(ValueType::F64) => Instruction::F64Eq,
                    None => Instruction::I32Const(1), // Unit Values are always equal
                };
                instructions.push(ins);
            }
            Op::Ne => {
                let ins = match reg_ty {
                    Some(ValueType::I32) => Instruction::I32Ne,
                    Some(ValueType::I64) => Instruction::I64Ne,
                    Some(ValueType::F32) => Instruction::F32Ne,
                    Some(ValueType::F64) => Instruction::F64Ne,
                    None => Instruction::I32Const(0), // Unit Values are never not equal
                };
                instructions.push(ins);
            }
            Op::Gt => {
                let ins = match (reg_ty.unwrap(), ty.is_uint()) {
                    (ValueType::I32, true) => Instruction::I32GtU,
                    (ValueType::I32, false) => Instruction::I32GtS,
                    (ValueType::I64, true) => Instruction::I64GtU,
                    (ValueType::I64, false) => Instruction::I64GtS,
                    (ValueType::F32, _) => Instruction::F32Gt,
                    (ValueType::F64, _) => Instruction::F64Gt,
                };
                instructions.push(ins);
            }
            Op::Ge => {
                let ins = match (reg_ty.unwrap(), ty.is_uint()) {
                    (ValueType::I32, true) => Instruction::I32GeU,
                    (ValueType::I32, false) => Instruction::I32GeS,
                    (ValueType::I64, true) => Instruction::I64GeU,
                    (ValueType::I64, false) => Instruction::I64GeS,
                    (ValueType::F32, _) => Instruction::F32Ge,
                    (ValueType::F64, _) => Instruction::F64Ge,
                };
                instructions.push(ins);
            }
            Op::Lt => {
                let ins = match (reg_ty.unwrap(), ty.is_uint()) {
                    (ValueType::I32, true) => Instruction::I32LtU,
                    (ValueType::I32, false) => Instruction::I32LtS,
                    (ValueType::I64, true) => Instruction::I64LtU,
                    (ValueType::I64, false) => Instruction::I64LtS,
                    (ValueType::F32, _) => Instruction::F32Lt,
                    (ValueType::F64, _) => Instruction::F64Lt,
                };
                instructions.push(ins);
            }
            Op::Le => {
                let ins = match (reg_ty.unwrap(), ty.is_uint()) {
                    (ValueType::I32, true) => Instruction::I32LeU,
                    (ValueType::I32, false) => Instruction::I32LeS,
                    (ValueType::I64, true) => Instruction::I64LeU,
                    (ValueType::I64, false) => Instruction::I64LeS,
                    (ValueType::F32, _) => Instruction::F32Le,
                    (ValueType::F64, _) => Instruction::F64Le,
                };
                instructions.push(ins);
            }
            Op::ConstInt(val) => {
                let ins = match reg_ty.unwrap() {
                    ValueType::I32 => Instruction::I32Const(*val as i32),
                    ValueType::I64 => Instruction::I64Const(*val),
                    ValueType::F32 => Instruction::F32Const((*val as f32).to_bits()),
                    ValueType::F64 => Instruction::F64Const((*val as f64).to_bits()),
                };
                instructions.push(ins);
            }
            Op::ConstFloat(val) => {
                let ins = match reg_ty.unwrap() {
                    ValueType::F32 => Instruction::F32Const((*val as f32).to_bits()),
                    ValueType::F64 => Instruction::F64Const(val.to_bits()),
                    _ => unreachable!(),
                };
                instructions.push(ins);
            }
            Op::ConstBool(val) => instructions.push(Instruction::I32Const(*val as i32)),
            Op::Drop => {
                if *ty != Ty::Unit {
                    instructions.push(Instruction::Drop);
                }
            }
            Op::LoadVar(i) => {
                let var = vars.get(entity).unwrap().0[*i];
                if let Some(Register(reg)) = registers.get(var) {
                    instructions.push(Instruction::GetLocal(*reg));
                }
            }
            Op::StoreVar(i) => {
                let var = vars.get(entity).unwrap().0[*i];
                if let Some(Register(reg)) = registers.get(var) {
                    instructions.push(Instruction::SetLocal(*reg));
                }
            }
            Op::StateVar(is_current, name) => {
                let index = state.lookup_index(name);
                instructions.push(Instruction::I32Const(index as i32));
                instructions.push(Instruction::I32Const(*is_current as i32));
                let ins = match ty {
                    Ty::U8 => 3,
                    Ty::U16 => 4,
                    Ty::U32 => 5,
                    Ty::U64 => 6,
                    Ty::I8 => 7,
                    Ty::I16 => 8,
                    Ty::I32 => 9,
                    Ty::I64 => 10,
                    Ty::F32 => 11,
                    Ty::F64 => 12,
                    _ => panic!("Unsupported state variable type"),
                };
                instructions.push(Instruction::Call(ins));
            }
            Op::If => {
                let block_ty = if let Some(reg_ty) = reg_ty {
                    BlockType::Value(reg_ty)
                } else {
                    BlockType::NoResult
                };
                instructions.push(Instruction::If(block_ty));
            }
            Op::Else => instructions.push(Instruction::Else),
            Op::End => instructions.push(Instruction::End),
            Op::Block => {
                let block_ty = if let Some(reg_ty) = reg_ty {
                    BlockType::Value(reg_ty)
                } else {
                    BlockType::NoResult
                };
                instructions.push(Instruction::Block(block_ty));
            }
            Op::Loop => {
                let block_ty = if let Some(reg_ty) = reg_ty {
                    BlockType::Value(reg_ty)
                } else {
                    BlockType::NoResult
                };
                instructions.push(Instruction::Loop(block_ty));
            }
            Op::Br(count) => instructions.push(Instruction::Br(*count)),
            Op::BrIf(count) => instructions.push(Instruction::BrIf(*count)),
            Op::Cast(expr) => {
                let from_ty = types.get(*expr).unwrap();
                lower_cast(
                    instructions,
                    from_ty,
                    ty,
                    needs_extending.get(*expr).is_some(),
                );
            }
            Op::Extend(entity) => {
                if needs_extending.get(*entity).is_some() {
                    lower_extend(instructions, ty);
                }
            }
            Op::ExtendVar(var_id) => {
                let var = vars.get(entity).unwrap().0[*var_id];
                if needs_extending.get(var).is_some() {
                    lower_extend(instructions, ty);
                }
            }
            Op::Call(fn_var_id) => {
                let fn_entity = vars.get(entity).unwrap().0[*fn_var_id];
                let FunctionIndex(fn_idx) = function_indices.get(fn_entity).unwrap();
                instructions.push(Instruction::Call(*fn_idx));
            }
        }
    }
}

fn lower_cast(
    instructions: &mut Vec<Instruction>,
    from_ty: &Ty,
    to_ty: &Ty,
    needs_extending: bool,
) {
    match (from_ty, to_ty) {
        // Integer upcasts
        (Ty::U8, Ty::U16) | (Ty::U8, Ty::U32) | (Ty::U8, Ty::I16) | (Ty::U8, Ty::I32) => {
            if needs_extending {
                lower_zext(instructions, 0xFF);
            }
        }
        (Ty::U8, Ty::U64) | (Ty::U8, Ty::I64) => {
            if needs_extending {
                lower_zext(instructions, 0xFF);
            }
            instructions.push(Instruction::I64ExtendUI32);
        }
        (Ty::I8, Ty::U16) | (Ty::I8, Ty::U32) | (Ty::I8, Ty::I16) | (Ty::I8, Ty::I32) => {
            if needs_extending {
                lower_sext(instructions, 24);
            }
        }
        (Ty::I8, Ty::U64) | (Ty::I8, Ty::I64) => {
            if needs_extending {
                lower_sext(instructions, 24);
            }
            instructions.push(Instruction::I64ExtendSI32);
        }
        (Ty::U16, Ty::U32) | (Ty::U16, Ty::I32) => {
            if needs_extending {
                lower_zext(instructions, 0xFFFF);
            }
        }
        (Ty::U16, Ty::U64) | (Ty::U16, Ty::I64) => {
            if needs_extending {
                lower_zext(instructions, 0xFFFF);
            }
            instructions.push(Instruction::I64ExtendUI32);
        }
        (Ty::I16, Ty::U32) | (Ty::I16, Ty::I32) => {
            if needs_extending {
                lower_sext(instructions, 16);
            }
        }
        (Ty::I16, Ty::U64) | (Ty::I16, Ty::I64) => {
            if needs_extending {
                lower_sext(instructions, 16);
            }
            instructions.push(Instruction::I64ExtendSI32);
        }
        (Ty::I32, Ty::U64) | (Ty::I32, Ty::I64) => {
            instructions.push(Instruction::I64ExtendSI32);
        }
        (Ty::U32, Ty::U64) | (Ty::U32, Ty::I64) | (Ty::Bool, Ty::U64) | (Ty::Bool, Ty::I64) => {
            instructions.push(Instruction::I64ExtendUI32);
        }
        // Integer downcasts
        (Ty::U16, Ty::U8)
        | (Ty::U16, Ty::I8)
        | (Ty::I16, Ty::U8)
        | (Ty::I16, Ty::I8)
        | (Ty::U32, Ty::U8)
        | (Ty::U32, Ty::U16)
        | (Ty::U32, Ty::I8)
        | (Ty::U32, Ty::I16)
        | (Ty::I32, Ty::U8)
        | (Ty::I32, Ty::U16)
        | (Ty::I32, Ty::I8)
        | (Ty::I32, Ty::I16) => {
            // We just keep the garbage bits. Only when upcasting or for certain
            // operations we zext / sext them away.
        }
        (Ty::U64, Ty::U8)
        | (Ty::U64, Ty::I8)
        | (Ty::U64, Ty::U16)
        | (Ty::U64, Ty::I16)
        | (Ty::I64, Ty::U8)
        | (Ty::I64, Ty::I8)
        | (Ty::I64, Ty::U16)
        | (Ty::I64, Ty::I16)
        | (Ty::U64, Ty::U32)
        | (Ty::U64, Ty::I32)
        | (Ty::I64, Ty::U32)
        | (Ty::I64, Ty::I32) => {
            instructions.push(Instruction::I32WrapI64);
        }
        // Integer sign casts
        (Ty::I8, Ty::U8)
        | (Ty::U8, Ty::I8)
        | (Ty::I16, Ty::U16)
        | (Ty::U16, Ty::I16)
        | (Ty::I32, Ty::U32)
        | (Ty::U32, Ty::I32)
        | (Ty::I64, Ty::U64)
        | (Ty::U64, Ty::I64) => {
            // Nothing to do
        }
        // Boolean casts
        (Ty::Bool, Ty::U8)
        | (Ty::Bool, Ty::U16)
        | (Ty::Bool, Ty::U32)
        | (Ty::Bool, Ty::I8)
        | (Ty::Bool, Ty::I16)
        | (Ty::Bool, Ty::I32) => {
            // No need to cast booleans into the same register size
        }
        // Integer to float casts
        (Ty::U8, Ty::F32) | (Ty::U16, Ty::F32) | (Ty::U32, Ty::F32) => {
            lower_cast(instructions, from_ty, &Ty::U32, needs_extending);
            instructions.push(Instruction::F32ConvertUI32);
        }
        (Ty::I8, Ty::F32) | (Ty::I16, Ty::F32) | (Ty::I32, Ty::F32) => {
            lower_cast(instructions, from_ty, &Ty::I32, needs_extending);
            instructions.push(Instruction::F32ConvertSI32);
        }
        (Ty::U8, Ty::F64) | (Ty::U16, Ty::F64) | (Ty::U32, Ty::F64) => {
            lower_cast(instructions, from_ty, &Ty::U32, needs_extending);
            instructions.push(Instruction::F64ConvertUI32);
        }
        (Ty::I8, Ty::F64) | (Ty::I16, Ty::F64) | (Ty::I32, Ty::F64) => {
            lower_cast(instructions, from_ty, &Ty::I32, needs_extending);
            instructions.push(Instruction::F64ConvertSI32);
        }
        (Ty::U64, Ty::F64) => instructions.push(Instruction::F64ConvertUI64),
        (Ty::I64, Ty::F64) => instructions.push(Instruction::F64ConvertSI64),
        // Float to integer casts
        (Ty::F32, Ty::U8) | (Ty::F32, Ty::U16) | (Ty::F32, Ty::U32) => {
            instructions.push(Instruction::I32TruncUF32);
        }
        (Ty::F32, Ty::I8) | (Ty::F32, Ty::I16) | (Ty::F32, Ty::I32) => {
            instructions.push(Instruction::I32TruncSF32);
        }
        (Ty::F64, Ty::U8) | (Ty::F64, Ty::U16) | (Ty::F64, Ty::U32) => {
            instructions.push(Instruction::I32TruncUF64);
        }
        (Ty::F64, Ty::I8) | (Ty::F64, Ty::I16) | (Ty::F64, Ty::I32) => {
            instructions.push(Instruction::I32TruncSF64);
        }
        (Ty::F64, Ty::U64) => {
            instructions.push(Instruction::I64TruncUF64);
        }
        (Ty::F64, Ty::I64) => {
            instructions.push(Instruction::I64TruncSF64);
        }
        (Ty::F32, Ty::U64) => {
            instructions.push(Instruction::I64TruncUF32);
        }
        (Ty::F32, Ty::I64) => {
            instructions.push(Instruction::I64TruncSF32);
        }
        // Float to float casts
        (Ty::F32, Ty::F64) => {
            instructions.push(Instruction::F64PromoteF32);
        }
        (Ty::F64, Ty::F32) => {
            instructions.push(Instruction::F32DemoteF64);
        }
        (a, b) if a == b => {}
        _ => panic!("Can't cast '{:?}' to '{:?}'", from_ty, to_ty),
    }
}

fn lower_extend(instructions: &mut Vec<Instruction>, ty: &Ty) {
    match ty {
        Ty::U8 => lower_zext(instructions, 0xFF),
        Ty::I8 => lower_sext(instructions, 24),
        Ty::U16 => lower_zext(instructions, 0xFFFF),
        Ty::I16 => lower_sext(instructions, 16),
        _ => {}
    }
}

fn lower_zext(instructions: &mut Vec<Instruction>, mask: i32) {
    instructions.push(Instruction::I32Const(mask));
    instructions.push(Instruction::I32And);
}

fn lower_sext(instructions: &mut Vec<Instruction>, bits: i32) {
    instructions.push(Instruction::I32Const(bits));
    instructions.push(Instruction::I32Shl);
    instructions.push(Instruction::I32Const(bits));
    instructions.push(Instruction::I32ShrS);
}
