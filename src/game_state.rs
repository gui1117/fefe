pub trait GameState {
    /// Return new state and true if next state gui must be set
    // fn gui(self, world: &mut ::specs::World) -> (Box<Self>, bool);
    // TODO: maybe put only ui and gameevents in args put whatever
    fn winit_event(self: Box<Self>, event: ::winit::Event, world: &mut ::specs::World) -> Box<GameState>;
    fn gilrs_event(self: Box<Self>, event: ::gilrs::EventType, world: &mut ::specs::World) -> Box<GameState>;
    fn quit(&self) -> bool;
    fn paused(&self) -> bool;
}

pub struct Game;

pub struct Menu;

impl GameState for Menu {
    fn winit_event(self: Box<Self>, event: ::winit::Event, world: &mut ::specs::World) -> Box<GameState> {
        self
    }

    fn gilrs_event(self: Box<Self>, event: ::gilrs::EventType, world: &mut ::specs::World) -> Box<GameState> {
        self
    }

    fn quit(&self) -> bool {
        false
    }
    fn paused(&self) -> bool {
        false
    }
}
