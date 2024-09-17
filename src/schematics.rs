use crate::{Schematic, SchematicContext, SchematicResult};

#[derive(Debug, Clone, Copy)]
pub enum Maybe<S> {
    Some(S),
    None,
}

impl<S> Schematic for Maybe<S>
where
    S: Schematic,
{
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult {
        match self {
            Maybe::Some(schematic) => schematic.instantiate(ctx),
            Maybe::None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrDefault<S>(pub Option<S>);

impl<S> Schematic for OrDefault<S>
where
    S: Schematic + Default,
{
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult {
        let schematic = match self.0 {
            Some(schematic) => schematic,
            None => S::default(),
        };

        schematic.instantiate(ctx)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Many<T>(pub T);

macro_rules! impl_many_tuple {
    ($($id: ident),+) => {
        impl<$($id),+> Schematic for Many<($($id),+)>
        where
            $($id: Schematic),+
        {
            #[allow(non_snake_case)]
            fn instantiate(self, mut ctx: &mut SchematicContext) -> SchematicResult {
                let Many(($($id),+)) = self;

                $($id.instantiate(&mut ctx)?;)+

                Ok(())
            }
        }
    };
}

impl_many_tuple!(A, B);
impl_many_tuple!(A, B, C);
impl_many_tuple!(A, B, C, D);
impl_many_tuple!(A, B, C, D, E);
impl_many_tuple!(A, B, C, D, E, F);
impl_many_tuple!(A, B, C, D, E, F, G);
impl_many_tuple!(A, B, C, D, E, F, G, H);
impl_many_tuple!(A, B, C, D, E, F, G, H, I);
impl_many_tuple!(A, B, C, D, E, F, G, H, I, J);
