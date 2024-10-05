use bevy_ecs::prelude::*;
use bevy_mod_schematica::prelude::*;

#[derive(Debug, Component, Clone)]
pub struct A;

#[derive(Debug, Component, Clone)]
pub struct B {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Component, Clone)]
pub struct C(pub u8);

#[derive(Debug, Component, Clone, Default)]
pub struct D(pub u8);

#[derive(Schematic)]
pub struct Simple {
    pub a: A,
    pub b: B,
    pub c: Children<Many<(C, OrDefault<D>)>>,
}

fn main() {
    let mut world = World::new();

    let simple = Simple {
        a: A,
        b: B { x: 4, y: 6 },
        c: Children(vec![
            Many((C(0), OrDefault::Some(D(10)))),
            Many((C(1), OrDefault::Default)),
            Many((C(2), OrDefault::Default)),
        ]),
    };

    let simple_entity = world.spawn_schematic(simple).expect("failed to run");

    println!(
        "Parent B: {:?}",
        world.get::<B>(simple_entity).expect("no B")
    );

    let children = world
        .get::<bevy_hierarchy::Children>(simple_entity)
        .expect("no children");

    for child in children.iter() {
        if let Some(c) = world.get::<C>(*child) {
            println!("{child} C: {c:?}");
        }

        if let Some(d) = world.get::<D>(*child) {
            println!("{child} D: {d:?}");
        }
    }
}
