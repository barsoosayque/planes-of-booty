use specs::{Component, NullStorage};

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct PendingDestruction;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Container;
