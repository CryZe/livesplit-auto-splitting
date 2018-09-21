use ast::{Children, Scoped, Source};
use debug_info::SrcByteRange;
use error::{RangeError, RangeResult, ResultExt};
use specs::prelude::*;
use types::Ty;

#[derive(Component)]
pub struct VarNames(pub Vec<String>);

#[derive(Component)]
pub struct VarDecl(pub usize);

#[derive(Component)]
pub struct Vars(pub Vec<Entity>);

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct DeclaredBy(pub Entity);

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct FunctionParamAsVar {
    pub function_var: usize,
    pub param_idx: usize,
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct FunctionDecl(pub usize, pub Vec<Entity>);

pub struct NameResolution<'s> {
    src: &'s Source,
    result: RangeResult<()>,
}

impl<'s> NameResolution<'s> {
    pub fn new(src: &'s Source) -> Self {
        Self {
            src,
            result: Ok(()),
        }
    }
    pub fn run(mut self, world: &World) -> RangeResult<()> {
        self.run_now(&world.res);
        self.result
    }
}

impl<'a, 's> System<'a> for NameResolution<'s> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Children>,
        ReadStorage<'a, Scoped>,
        ReadStorage<'a, VarNames>,
        ReadStorage<'a, VarDecl>,
        WriteStorage<'a, Vars>,
        WriteStorage<'a, Ty>,
        ReadStorage<'a, SrcByteRange>,
        ReadStorage<'a, FunctionDecl>,
        ReadStorage<'a, FunctionParamAsVar>,
        WriteStorage<'a, DeclaredBy>,
    );

    fn run(
        &mut self,
        (
            entities,
            children,
            scoped,
            var_names,
            var_decls,
            mut vars,
            mut types,
            ranges,
            function_decls,
            function_params,
            mut declared_bys,
        ): Self::SystemData,
    ) {
        let mut scopes = Scopes::default();

        for (entity, fn_names, function_decl) in (&*entities, &var_names, &function_decls).join() {
            let id = function_decl.0;
            scopes.declare_existing_var(&fn_names.0[id], entity);
        }

        for (_, fn_entity) in self.src.code_items() {
            if let Err(e) = resolve_names(
                &mut scopes,
                &entities,
                fn_entity,
                &children,
                &scoped,
                &var_names,
                &var_decls,
                &mut vars,
                &mut types,
                &function_params,
                &function_decls,
                &ranges,
                &mut declared_bys,
            ) {
                self.result = Err(e);
                return;
            }
        }
    }
}

fn resolve_names<'a>(
    scopes: &mut Scopes<'a>,
    entities: &Entities,
    entity: Entity,
    children: &ReadStorage<Children>,
    scoped: &ReadStorage<Scoped>,
    var_names: &'a ReadStorage<VarNames>,
    var_decls: &ReadStorage<VarDecl>,
    vars: &mut WriteStorage<Vars>,
    types: &mut WriteStorage<Ty>,
    function_params: &ReadStorage<FunctionParamAsVar>,
    function_decls: &ReadStorage<FunctionDecl>,
    ranges: &ReadStorage<SrcByteRange>,
    declared_bys: &mut WriteStorage<DeclaredBy>,
) -> RangeResult<()> {
    let introduces_scope = scoped.get(entity).is_some();
    if introduces_scope {
        scopes.push_scope();
    }

    if let Some(VarNames(list_of_names)) = var_names.get(entity) {
        if let Some(VarDecl(id)) = var_decls.get(entity) {
            let var_entity = scopes.declare_var(&list_of_names[*id], entities);
            declared_bys.insert(var_entity, DeclaredBy(entity)).unwrap();
            if let Some(ty) = types.get(entity).cloned() {
                types.insert(var_entity, ty).unwrap();
            }
        }

        let resolved_vars = list_of_names
            .iter()
            .map(|var_name| scopes.lookup(var_name))
            .collect::<RangeResult<_>>()
            .with_entity_range(entity, ranges)?;

        #[allow(never_loop)]
        loop {
            // TODO NLL
            if let Some(Vars(existing_vars)) = vars.get_mut(entity) {
                existing_vars.extend(resolved_vars);
                break;
            }
            vars.insert(entity, Vars(resolved_vars)).unwrap();
            break;
        }
    }

    if let Some(param) = function_params.get(entity) {
        let Vars(vars) = vars.get_mut(entity).unwrap();
        let function_entity = vars[param.function_var];
        let FunctionDecl(_, params) = function_decls.get(function_entity).unwrap();
        if let Some(param_var) = params.get(param.param_idx) {
            vars.push(*param_var);
        }
    }

    if let Some(my_children) = children.get(entity) {
        for child in &my_children.0 {
            resolve_names(
                scopes,
                entities,
                *child,
                children,
                scoped,
                var_names,
                var_decls,
                vars,
                types,
                function_params,
                function_decls,
                ranges,
                declared_bys,
            )?;
        }
    }

    if introduces_scope {
        scopes.pop_scope();
    }

    Ok(())
}

#[derive(Default)]
struct Scopes<'a> {
    frames: Vec<usize>,
    vars: Vec<(&'a str, Entity)>,
}

impl<'a> Scopes<'a> {
    fn push_scope(&mut self) {
        self.frames.push(self.vars.len());
    }

    fn pop_scope(&mut self) {
        let len = self.frames.pop().unwrap();
        self.vars.drain(len..);
    }

    fn lookup(&self, name: &str) -> RangeResult<Entity> {
        self.vars
            .iter()
            .rev()
            .filter(|(existing_name, _)| *existing_name == name)
            .map(|(_, entity)| *entity)
            .next()
            .ok_or_else(|| RangeError::new(format!("Variable '{}' is not in scope", name)))
    }

    fn declare_var(&mut self, name: &'a str, entities: &Entities) -> Entity {
        let entity = entities.create();
        self.vars.push((name, entity));
        entity
    }

    fn declare_existing_var(&mut self, name: &'a str, entity: Entity) {
        self.vars.push((name, entity));
    }
}
