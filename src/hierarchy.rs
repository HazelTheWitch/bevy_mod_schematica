use crate::{Schematic, SchematicContext, SchematicError, SchematicResult};

#[derive(Debug, Clone, Copy)]
pub struct Parent<S>(pub S);

impl<S> Schematic for Parent<S>
where
    S: Schematic,
{
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult {
        let Some(parent) = ctx.current().parent else {
            return Err(SchematicError("entity does not have parent"));
        };

        let mut parent_ctx = ctx.of(parent).unwrap_or_else(|| unreachable!());

        self.0.instantiate(&mut parent_ctx)
    }
}

#[derive(Debug, Clone)]
pub struct Children<S>(pub Vec<S>);

impl<S> Schematic for Children<S>
where
    S: Schematic,
{
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult {
        ctx.try_map_children(self.0.into_iter(), |mut ctx, s| s.instantiate(&mut ctx))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RepeatChildren<S> {
    pub schematic: S,
    pub count: usize,
}

impl<S> Schematic for RepeatChildren<S>
where
    S: Schematic + Clone,
{
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult {
        ctx.try_map_children(0..self.count, |mut ctx, _| {
            self.schematic.clone().instantiate(&mut ctx)
        })?;

        Ok(())
    }
}
