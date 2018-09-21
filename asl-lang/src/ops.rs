use ast::Scoped;
use ast::{self, ActionKind, Children};
use code_gen::{CodeGenDesc, Op};
use debug_info::ReferencesVar;
use debug_info::SrcByteRange;
use name_resolution::{FunctionDecl, VarDecl, VarNames};
use reg_extend::{ExtendConnection, InferExtending, NeedsExtending};
use specs::prelude::*;
use types::{Inference, Ty, TypeChecking};

pub fn bin_op_extend(left: Entity, right: Entity, op: Op) -> CodeGenDesc {
    CodeGenDesc(vec![
        Op::Entity(left),
        Op::Extend(left),
        Op::Entity(right),
        Op::Extend(right),
        op,
    ])
}

pub fn bin_op(left: Entity, right: Entity, op: Op) -> CodeGenDesc {
    CodeGenDesc(vec![Op::Entity(left), Op::Entity(right), op])
}

pub fn unary_op(val: Entity, op: Op) -> CodeGenDesc {
    CodeGenDesc(vec![Op::Entity(val), op])
}

pub fn build_action(
    world: &mut World,
    kind: ActionKind,
    ty: Ty,
    block: Entity,
    (l, r): (usize, usize),
) -> ast::Item {
    let entity = world
        .create_entity()
        .with(Children(vec![block]))
        .with(CodeGenDesc(vec![Op::Entity(block)]))
        .with(TypeChecking(vec![Inference::SameAsMe(block)]))
        .with(ty)
        .with(SrcByteRange(l, r))
        .build();

    ast::Item::Action(kind, entity)
}

pub fn build_compare(
    world: &mut World,
    l: Entity,
    r: Entity,
    op: Op,
    inner_ty: Option<Ty>,
    range: SrcByteRange,
) -> Entity {
    let inner = {
        let mut inner = world
            .create_entity()
            .with(Children(vec![l, r]))
            .with(bin_op_extend(l, r, op))
            .with(TypeChecking(vec![
                Inference::SameAsMe(l),
                Inference::SameAsMe(r),
            ]));

        if let Some(inner_ty) = inner_ty {
            inner = inner.with(inner_ty);
        }

        inner.build()
    };

    world
        .create_entity()
        .with(Children(vec![inner]))
        .with(CodeGenDesc(vec![Op::Entity(inner)]))
        .with(Ty::Bool)
        .with(range)
        .build()
}

pub fn build_op_assign(
    world: &mut World,
    (name, l, r): (String, usize, usize),
    expr: Entity,
    op: Op,
    ty: Ty,
    extend: bool,
    dirty: bool,
) -> Entity {
    let ops = if extend {
        vec![
            Op::LoadVar(0),
            Op::ExtendVar(0),
            Op::Entity(expr),
            Op::Extend(expr),
            op,
            Op::StoreVar(0),
        ]
    } else {
        vec![Op::LoadVar(0), Op::Entity(expr), op, Op::StoreVar(0)]
    };

    let mut builder = world
        .create_entity()
        .with(Children(vec![expr]))
        .with(VarNames(vec![name]))
        .with(TypeChecking(vec![
            Inference::SameAsMe(expr),
            Inference::VarSameAsMe(0),
        ])).with(ty)
        .with(CodeGenDesc(ops))
        .with(SrcByteRange(l, r))
        .with(ReferencesVar(0))
        .with(InferExtending(vec![
            ExtendConnection::FromEntity(expr),
            ExtendConnection::StoreVar(0),
        ]));

    if dirty {
        builder = builder.with(NeedsExtending);
    }

    builder.build()
}

pub fn build_fn(
    world: &mut World,
    fn_name: String,
    params: Vec<(String, Option<Ty>, SrcByteRange)>,
    ty: Option<Ty>,
    block: Entity,
    (l, r): (usize, usize),
) -> ast::Item {
    let mut param_entities = Vec::new();

    for (name, ty, range) in params {
        let mut builder = world
            .create_entity()
            .with(VarNames(vec![name]))
            .with(VarDecl(0))
            .with(ReferencesVar(0))
            .with(TypeChecking(vec![Inference::VarSameAsMe(0)]))
            .with(range);

        if let Some(ty) = ty {
            builder = builder.with(ty);
        }

        param_entities.push(builder.build());
    }

    let mut children = param_entities.clone();
    children.push(block);

    let mut entity = world
        .create_entity()
        .with(Children(children))
        .with(SrcByteRange(l, r))
        .with(VarNames(vec![fn_name]))
        .with(ReferencesVar(0))
        .with(FunctionDecl(0, param_entities))
        .with(CodeGenDesc(vec![Op::Entity(block)]))
        .with(TypeChecking(vec![Inference::SameAsMe(block)]))
        .with(Scoped)
        .with(InferExtending(vec![
            ExtendConnection::FromEntity(block),
            ExtendConnection::StoreVar(0),
        ]));

    if let Some(ty) = ty {
        entity = entity.with(ty);
    }

    ast::Item::Function(entity.build())
}
