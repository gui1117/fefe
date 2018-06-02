use specs::World;

pub fn build(ui: &::imgui::Ui, _world: &World) {
    ui.window(im_str!("Configuration menu")).build(|| {});
}
