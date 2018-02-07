pub trait GameState {
    /// Return new state and true if next state gui must be set
    // fn gui(self, world: &mut ::specs::World) -> (Box<Self>, bool);
    // or maybe it can be put in world ?
    fn winit_event(
        self: Box<Self>,
        event: ::winit::Event,
        world: &mut ::specs::World,
    ) -> Box<GameState>;
    fn gilrs_event(
        self: Box<Self>,
        event: ::gilrs::EventType,
        world: &mut ::specs::World,
    ) -> Box<GameState>;
    fn quit(&self) -> bool {
        false
    }
    fn paused(&self) -> bool;
}

pub struct Game;

impl GameState for Game {
    fn winit_event(
        self: Box<Self>,
        _event: ::winit::Event,
        _world: &mut ::specs::World,
    ) -> Box<GameState> {
        self
    }

    fn gilrs_event(
        self: Box<Self>,
        _event: ::gilrs::EventType,
        _world: &mut ::specs::World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self) -> bool {
        false
    }
}
