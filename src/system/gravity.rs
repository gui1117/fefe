use specs::Join;

pub struct GravitySystem;

impl<'a> ::specs::System<'a> for GravitySystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::GravityToPlayers>,
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::WriteStorage<'a, ::component::RigidBody>,
        ::specs::FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            gravities,
            players,
            mut rigid_bodies,
            mut physic_world,
        ): Self::SystemData,
    ) {
        let players = (&players, &rigid_bodies).join()
            .map(|(_, body)| body.get(&physic_world).center_of_mass())
            .collect::<Vec<_>>();

        for (gravity, body) in (&gravities, &mut rigid_bodies).join() {
            let body = body.get_mut(&mut physic_world);
            let _forces = players.iter()
                .map(|p| {
                    let dir = p - body.center_of_mass();
                    let distance = dir.norm();

                    let vec = ::CFG.gravity*gravity.mass/distance.powi(3) * dir;
                    assert_eq!(vec, vec);
                })
                .collect::<Vec<_>>();

            // TODO: apply force
        }
    }
}
