use name_resolution::DeclaredBy;
use name_resolution::FunctionDecl;
use name_resolution::Vars;
use specs::prelude::*;
use types::Ty;

#[derive(Copy, Clone, Debug, Component)]
#[storage(DenseVecStorage)]
pub struct SrcByteRange(pub usize, pub usize);

impl SrcByteRange {
    pub fn to_span(&self, src: &str) -> Span {
        Span {
            from: line_column(src, self.0),
            to: line_column(src, self.1),
        }
    }
}

#[derive(Component)]
pub struct ReferencesVar(pub usize);

fn line_column(src: &str, byte_pos: usize) -> (usize, usize) {
    let (mut total_bytes, mut line, mut column) = (0, 1, 1);
    for row in src.split_terminator('\n') {
        let new_total_bytes = total_bytes + row.len() + 1;
        if byte_pos < new_total_bytes {
            column = byte_pos - total_bytes + 1;
            break;
        } else {
            total_bytes = new_total_bytes;
        }
        line += 1;
    }
    (line, column)
}

#[derive(Debug, PartialEq)]
pub struct Span {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

impl Span {
    pub fn from_byte_position(src: &str, byte_pos: usize) -> Self {
        let (l, c) = line_column(src, byte_pos);
        Self {
            from: (l, c),
            to: (l, c + 1),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Hover {
    pub entity: Entity,
    pub params: Option<Vec<Ty>>,
    pub ty: Ty,
    pub span: Span,
}

pub struct HoverSystem<'s> {
    line: usize,
    column: usize,
    result: Option<Hover>,
    src: &'s str,
}

impl<'s> HoverSystem<'s> {
    pub fn new(src: &'s str, line: usize, column: usize) -> Self {
        Self {
            src,
            line,
            column,
            result: None,
        }
    }

    pub fn run(mut self, world: &World) -> Option<Hover> {
        self.run_now(&world.res);
        self.result
    }

    fn byte_pos(&self) -> usize {
        let mut bytes = 0;
        for (l, row) in self.src.split_terminator('\n').take(self.line).enumerate() {
            if l + 1 < self.line {
                bytes += row.len() + 1;
            } else {
                bytes += self.column - 1;
            }
        }
        bytes
    }
}

impl<'a, 's> System<'a> for HoverSystem<'s> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, SrcByteRange>,
        ReadStorage<'a, Ty>,
        ReadStorage<'a, FunctionDecl>,
    );

    fn run(&mut self, (entities, ranges, types, function_decls): Self::SystemData) {
        let byte_pos = self.byte_pos();
        let (mut min_ty, mut min_from, mut min_len) = (None, 0, usize::max_value());
        for (entity, SrcByteRange(from, to), ty) in (&*entities, &ranges, &types).join() {
            if byte_pos >= *from && byte_pos < *to {
                let len = *to - *from;
                if len < min_len {
                    min_ty = Some((ty, entity));
                    min_from = *from;
                    min_len = len;
                }
            }
        }
        if let Some((ty, entity)) = min_ty {
            let params = function_decls.get(entity).map(|FunctionDecl(_, params)| {
                params
                    .iter()
                    .filter_map(|p| types.get(*p).cloned())
                    .collect()
            });

            self.result = Some(Hover {
                entity,
                params,
                ty: ty.clone(),
                span: SrcByteRange(min_from, min_from + min_len).to_span(self.src),
            });
        }
    }
}

pub struct GoToDefinition {
    hover: Hover,
    result: Option<(Entity, SrcByteRange)>,
}

impl GoToDefinition {
    pub fn run(src: &str, world: &World, line: usize, column: usize) -> Option<(Entity, Span)> {
        let hover = HoverSystem::new(src, line, column).run(world)?;
        let mut system = Self {
            hover,
            result: None,
        };
        system.run_now(&world.res);
        system.result.map(|(e, s)| (e, s.to_span(src)))
    }
}

impl<'a> System<'a> for GoToDefinition {
    type SystemData = (
        ReadStorage<'a, ReferencesVar>,
        ReadStorage<'a, Vars>,
        ReadStorage<'a, SrcByteRange>,
        ReadStorage<'a, DeclaredBy>,
    );

    fn run(&mut self, (references_vars, vars, ranges, declared_bys): Self::SystemData) {
        if let (Some(ReferencesVar(var_index)), Some(Vars(vars))) = (
            references_vars.get(self.hover.entity),
            vars.get(self.hover.entity),
        ) {
            let var = vars[*var_index];
            self.result = ranges.get(var).map(|r| (var, *r));
            if self.result.is_none() {
                if let Some(DeclaredBy(by)) = declared_bys.get(var) {
                    self.result = ranges.get(*by).map(|r| (*by, *r));
                }
            }
        }
    }
}

pub struct FindAllVariableReferences<'s> {
    src: &'s str,
    hover_entity: Entity,
    result: Option<Vec<Span>>,
}

impl<'s> FindAllVariableReferences<'s> {
    pub fn run(src: &'s str, world: &World, line: usize, column: usize) -> Option<Vec<Span>> {
        let hover = HoverSystem::new(src, line, column).run(world)?;
        let mut system = Self {
            src,
            hover_entity: hover.entity,
            result: None,
        };
        system.run_now(&world.res);
        system.result
    }
}

impl<'a, 's> System<'a> for FindAllVariableReferences<'s> {
    type SystemData = (
        ReadStorage<'a, Vars>,
        ReadStorage<'a, ReferencesVar>,
        ReadStorage<'a, SrcByteRange>,
    );

    fn run(&mut self, (vars, references_vars, ranges): Self::SystemData) {
        let search_var = if let (Some(ReferencesVar(var_index)), Some(Vars(vars))) = (
            references_vars.get(self.hover_entity),
            vars.get(self.hover_entity),
        ) {
            vars[*var_index]
        } else {
            return;
        };

        let mut spans = Vec::new();

        for (Vars(vars), ReferencesVar(var_index), range) in
            (&vars, &references_vars, &ranges).join()
        {
            let referenced = vars[*var_index];
            if referenced == search_var {
                spans.push(range.to_span(self.src));
            }
        }

        self.result = Some(spans);
    }
}
