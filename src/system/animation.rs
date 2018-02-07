use specs::Join;

pub struct AnimationSystem;

impl<'a> ::specs::System<'a> for AnimationSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::RigidBody>,
        ::specs::WriteStorage<'a, ::component::AnimationState>,
        ::specs::Fetch<'a, ::resource::UpdateTime>,
        ::specs::Fetch<'a, ::resource::PhysicWorld>,
        ::specs::FetchMut<'a, ::resource::AnimationImages>,
    );

    fn run(
        &mut self,
        (
            rigid_bodies,
            mut animation_states,
            update_time,
            physic_world,
            mut animation_images,
        ): Self::SystemData,
){
        for (state, body) in (&mut animation_states, &rigid_bodies).join() {
            let body = body.get(&physic_world);

            let velocity = body.velocity().linear.norm();
            if velocity <= ::std::f32::EPSILON {
                state.distance = 0.0;
            } else {
                state.distance += update_time.0 * velocity;
            }

            state.timer += update_time.0;

            // Remove finished animations
            loop {
                {
                    let animation = state.animations.first();
                    match animation {
                        Some(animation) => if state.timer >= animation.duration {
                            state.timer -= animation.duration;
                        } else {
                            break;
                        },
                        None => break,
                    }
                }
                state.animations.remove(0);
            }

            let animation = state.animations.first().unwrap_or(&state.idle_animation);

            for part in &animation.parts {
                animation_images.push(::animation::AnimationImage {
                    position: body.position(),
                    layer: part.layer,
                    id: part.image_at(state.timer, state.distance),
                });
            }
        }
    }
}
