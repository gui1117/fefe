use specs::World;

pub fn build(ui: &::imgui::Ui, world: &World) {
    ui.window(im_str!("Configuration menu"))
        .build(|| {
        });
}
