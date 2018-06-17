use specs::Join;

pub struct AudioSystem;

impl<'a> ::specs::System<'a> for AudioSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::ReadStorage<'a, ::component::RigidBody>,
        ::specs::ReadExpect<'a, ::resource::PhysicWorld>,
        ::specs::ReadExpect<'a, ::resource::Save>,
        ::specs::ReadExpect<'a, ::resource::Conf>,
        ::specs::WriteExpect<'a, ::resource::Audio>,
    );

    fn run(
        &mut self,
        (players, bodies, physic_world, save, conf, mut audio): Self::SystemData,
    ) {
        // TODO: Fix it when multiple bodies
        let position = (&players, &bodies).join().next().unwrap().1
            .get(&physic_world)
            .position()
            .translation.vector;
        audio.update(position, conf.audio_z_distance, conf.audio_ear_distance, save.effect_volume);
    }
}
