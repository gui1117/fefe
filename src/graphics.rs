use alga::general::SubsetOf;
use ncollide2d::shape::{self, ShapeHandle};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, ImmutableBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::{
    DescriptorSet, FixedSizeDescriptorSetsPool, PersistentDescriptorSet,
};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::{self, ClearValue, Format};
use vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, LayoutAttachmentDescription, LayoutPassDependencyDescription,
    LayoutPassDescription, LoadOp, RenderPassAbstract, RenderPassDesc, RenderPassDescClearValues,
    StoreOp, Subpass,
};
use vulkano::image::ImageLayout;
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, ImmutableImage};
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::swapchain::{self, AcquireError, Surface, Swapchain, SwapchainCreationError};
use vulkano::sync::{now, FlushError, GpuFuture};
use itertools::Itertools;
use specs::Join;
use specs::World;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

// TODO: only a bool for whereas draw the cursor or not

const DEBUG_SEGMENT_WIDTH: f32 = 1.0;

pub struct Camera {
    pub position: ::na::Isometry2<f32>,
    pub zoom: f32,
}

impl Camera {
    pub fn new(position: ::na::Isometry2<f32>, zoom: f32) -> Self {
        Camera { position, zoom }
    }
}

impl Camera {
    fn matrix(&self, dimensions: [u32; 2]) -> [[f32; 4]; 4] {
        let rescale_trans = {
            let ratio = dimensions[0] as f32 / dimensions[1] as f32;

            let (kx, ky) = if ratio > 1. {
                (1.0 / (self.zoom * ratio), 1.0 / self.zoom)
            } else {
                (1.0 / self.zoom, ratio / self.zoom)
            };

            let mut trans: ::na::Transform3<f32> = ::na::one();
            trans[(0, 0)] = kx;
            trans[(1, 1)] = ky;
            trans
        };

        let world_trans: ::na::Transform3<f32> = ::na::Isometry3::<f32>::new(
            ::na::Vector3::new(
                self.position.translation.vector[0],
                -self.position.translation.vector[1],
                0.0,
            ),
            ::na::Vector3::new(0.0, 0.0, -self.position.rotation.angle()),
        ).inverse()
            .to_superset();

        (rescale_trans * world_trans).unwrap().into()
    }
}

struct Image {
    descriptor_set: Arc<DescriptorSet + Sync + Send>,
    width: u32,
    height: u32,
}

pub struct Graphics {
    queue: Arc<Queue>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain<::winit::Window>>,
    render_pass: Arc<RenderPassAbstract + Sync + Send>,
    pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    debug_pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    animation_images: Vec<Image>,
    framebuffers: Vec<Arc<FramebufferAbstract + Sync + Send>>,
    view_buffer_pool: CpuBufferPool<vs::ty::View>,
    world_buffer_pool: CpuBufferPool<vs::ty::World>,
    descriptor_sets_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Sync + Send>>,
    future: Option<Box<GpuFuture>>,

    imgui_pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    imgui_descriptor_set: Arc<DescriptorSet + Send + Sync>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

#[derive(Debug, Clone)]
struct DebugVertex {
    position: [f32; 2],
    color: [f32; 4],
}
impl_vertex!(DebugVertex, position, color);

#[derive(Debug, Clone)]
pub struct ImGuiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    col: [f32; 4],
}

impl_vertex!(ImGuiVertex, pos, uv, col);
impl From<::imgui::ImDrawVert> for ImGuiVertex {
    fn from(vertex: ::imgui::ImDrawVert) -> Self {
        let r = vertex.col as u8 as f32;
        let g = (vertex.col >> 8) as u8 as f32;
        let b = (vertex.col >> 16) as u8 as f32;
        let a = (vertex.col >> 24) as u8 as f32;
        ImGuiVertex {
            pos: [vertex.pos.x, vertex.pos.y],
            uv: [vertex.uv.x, vertex.uv.y],
            col: [r, g, b, a],
        }
    }
}
impl Graphics {
    pub fn new(window: &Arc<Surface<::winit::Window>>, imgui: &mut ::imgui::ImGui) -> Graphics {
        let physical = PhysicalDevice::enumerate(&window.instance())
            .next()
            .expect("no device available");

        let queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics()
                    && q.supports_compute()
                    && window.is_supported(q).unwrap_or(false)
            })
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            };

            Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue_family, 0.5)].iter().cloned(),
            ).expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let caps = window
                .capabilities(physical)
                .expect("failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let format = caps.supported_formats[0].0;
            let image_usage = ImageUsage {
                color_attachment: true,
                ..ImageUsage::none()
            };

            Swapchain::new(
                device.clone(),
                window.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                image_usage,
                &queue,
                swapchain::SurfaceTransform::Identity,
                swapchain::CompositeAlpha::Opaque,
                swapchain::PresentMode::Fifo,
                true,
                None,
            ).expect("failed to create swapchain")
        };

        let render_pass = Arc::new(
            CustomRenderPassDesc {
                swapchain_image_format: swapchain.format(),
            }.build_render_pass(device.clone())
                .unwrap(),
        );

        let mut future = Box::new(now(device.clone())) as Box<GpuFuture>;

        let (vertex_buffer, vertex_buffer_fut) = ImmutableBuffer::from_iter(
            [
                [-0.5f32, -0.5],
                [-0.5, 0.5],
                [0.5, -0.5],
                [0.5, 0.5],
                [-0.5, 0.5],
                [0.5, -0.5],
            ].iter()
                .cloned()
                .map(|position| Vertex { position }),
            BufferUsage::vertex_buffer(),
            queue.clone(),
        ).expect("failed to create buffer");
        future = Box::new(future.join(vertex_buffer_fut)) as Box<_>;

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let debug_vs =
            debug_vs::Shader::load(device.clone()).expect("failed to create shader module");
        let debug_fs =
            debug_fs::Shader::load(device.clone()).expect("failed to create shader module");

        let imgui_vs =
            imgui_vs::Shader::load(device.clone()).expect("failed to create shader module");
        let imgui_fs =
            imgui_fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_dynamic_scissors_irrelevant(1)
                .cull_mode_back()
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let debug_pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<DebugVertex>()
                .vertex_shader(debug_vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(debug_fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let imgui_pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<ImGuiVertex>()
                .vertex_shader(imgui_vs.main_entry_point(), ())
                .triangle_list()
                .cull_mode_front()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(imgui_fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);

        let imgui_texture = imgui
            .prepare_texture(|handle| {
                ImmutableImage::from_iter(
                    handle.pixels.iter().cloned(),
                    Dimensions::Dim2d {
                        width: handle.width,
                        height: handle.height,
                    },
                    format::R8G8B8A8Unorm,
                    queue.clone(),
                )
            })
            .unwrap()
            .0;

        let imgui_descriptor_set = {
            Arc::new(
                PersistentDescriptorSet::start(imgui_pipeline.clone(), 0)
                    .add_sampled_image(
                        imgui_texture,
                        Sampler::new(
                            device.clone(),
                            Filter::Nearest,
                            Filter::Linear,
                            MipmapMode::Linear,
                            SamplerAddressMode::MirroredRepeat,
                            SamplerAddressMode::MirroredRepeat,
                            SamplerAddressMode::MirroredRepeat,
                            0.0,
                            1.0,
                            0.0,
                            0.0,
                        ).unwrap(),
                    )
                    .unwrap()
                    .build()
                    .unwrap(),
            )
        };

        let mut animation_images = vec![];

        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0,
            1.0,
            0.0,
            0.0,
        ).unwrap();

        for image_path in &::animation::ANIMATIONS.images {
            let file = File::open(image_path).unwrap();
            let (info, mut reader) = ::png::Decoder::new(file).read_info().unwrap();
            assert_eq!(info.color_type, ::png::ColorType::RGBA);
            let mut buf = vec![0; info.buffer_size()];
            reader.next_frame(&mut buf).unwrap();

            let (image, image_fut) = ImmutableImage::from_iter(
                buf.into_iter(),
                Dimensions::Dim2d {
                    width: info.width,
                    height: info.height,
                },
                format::R8G8B8A8Srgb,
                queue.clone(),
            ).unwrap();
            future = Box::new(future.join(image_fut)) as Box<_>;

            let descriptor_set = Arc::new(
                PersistentDescriptorSet::start(pipeline.clone(), 1)
                    .add_sampled_image(image.clone(), sampler.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<_>;

            animation_images.push(Image {
                descriptor_set,
                width: info.width,
                height: info.height,
            })
        }

        let view_buffer_pool =
            CpuBufferPool::<vs::ty::View>::new(device.clone(), BufferUsage::uniform_buffer());

        let world_buffer_pool =
            CpuBufferPool::<vs::ty::World>::new(device.clone(), BufferUsage::uniform_buffer());

        let depth_buffer_attachment = AttachmentImage::transient(
            device.clone(),
            images[0].dimensions(),
            format::Format::D16Unorm,
        ).unwrap();

        let framebuffers = images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer_attachment.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<_>
            })
            .collect::<Vec<_>>();

        let future = Some(Box::new(future.then_signal_fence_and_flush().unwrap()) as Box<_>);

        Graphics {
            device,
            future,
            queue,
            swapchain,
            render_pass,
            pipeline,
            debug_pipeline,
            vertex_buffer,
            animation_images,
            framebuffers,
            view_buffer_pool,
            world_buffer_pool,
            descriptor_sets_pool,
            imgui_pipeline,
            imgui_descriptor_set,
        }
    }

    fn recreate(&mut self, window: &Arc<Surface<::winit::Window>>) {
        let recreate;
        let mut remaining_try = 20;
        loop {
            let dimensions = window
                .capabilities(self.device.physical_device())
                .expect("failed to get surface capabilities")
                .current_extent
                .unwrap_or([1024, 768]);

            let res = self.swapchain.recreate_with_dimension(dimensions);

            if remaining_try == 0 {
                recreate = res;
                break;
            }

            match res {
                Err(SwapchainCreationError::UnsupportedDimensions) => (),
                res @ _ => {
                    recreate = res;
                    break;
                }
            }
            remaining_try -= 1;
            ::std::thread::sleep(::std::time::Duration::from_millis(50));
        }

        let (swapchain, images) = recreate.unwrap();
        self.swapchain = swapchain;

        // TODO: factorize
        let depth_buffer_attachment = AttachmentImage::transient(
            self.device.clone(),
            images[0].dimensions(),
            format::Format::D16Unorm,
        ).unwrap();

        self.framebuffers = images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(self.render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer_attachment.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<_>
            })
            .collect::<Vec<_>>();
    }

    fn build_command_buffer(
        &mut self,
        image_num: usize,
        world: &mut World,
    ) -> AutoCommandBuffer<StandardCommandPoolAlloc> {
        let dimensions = self.swapchain.dimensions();

        let screen_dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::none()
        };

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        ).unwrap()
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                false,
                vec![[0.0, 0.0, 0.0, 1.0].into(), 1.0.into()],
            )
            .unwrap();

        // Draw world
        let view = vs::ty::View {
            view: world
                .read_resource::<::resource::Camera>()
                .matrix(dimensions),
        };
        let view_buffer = self.view_buffer_pool.next(view).unwrap();

        let mut images = world.write_resource::<::resource::AnimationImages>();
        for image in images.drain(..) {
            let world_matrix: ::na::Transform3<f32> = ::na::Isometry3::<f32>::new(
                ::na::Vector3::new(
                    image.position.translation.vector[0],
                    -image.position.translation.vector[1],
                    0.0,
                ),
                ::na::Vector3::new(0.0, 0.0, -image.position.rotation.angle()),
            ).to_superset();

            let world = vs::ty::World {
                world: world_matrix.unwrap().into(),
            };
            let world_buffer = self.world_buffer_pool.next(world).unwrap();

            let sets = self.descriptor_sets_pool
                .next()
                .add_buffer(view_buffer.clone())
                .unwrap()
                .add_buffer(world_buffer)
                .unwrap()
                .build()
                .unwrap();

            command_buffer_builder = command_buffer_builder
                .draw(
                    self.pipeline.clone(),
                    screen_dynamic_state.clone(),
                    vec![self.vertex_buffer.clone()],
                    (sets, self.animation_images[image.id].descriptor_set.clone()),
                    vs::ty::Info {
                        layer: image.layer,
                        height: self.animation_images[image.id].height as f32,
                        width: self.animation_images[image.id].width as f32,
                    },
                )
                .unwrap()
        }

        // Draw physic world
        if true {
            let sets = Arc::new(
                PersistentDescriptorSet::start(self.debug_pipeline.clone(), 0)
                    .add_buffer(view_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            let bodies_map = world.read_resource::<::resource::BodiesMap>();
            let debug_colors = world.read::<::component::DebugColor>();
            let debug_circles = world.read::<::component::DebugCircles>();
            let bodies = world.read::<::component::RigidBody>();
            let physic_world = world.read_resource::<::resource::PhysicWorld>();
            let debug_shapes= world.read_resource::<::resource::DebugShapes>();

            let mut vertices = vec![];
            for collider in physic_world.colliders() {
                let color = bodies_map.get(&collider.data().body())
                    .and_then(|e| debug_colors.get(*e))
                    .map(|c| c.0)
                    .unwrap_or(0);
                shape_vertices(collider.position(), collider.shape(), COLORS[color], &mut vertices);
            }
            for (radiuss, color, body) in (&debug_circles, &debug_colors, &bodies).join() {
                let body = body.get(&physic_world);
                for &radius in radiuss.iter() {
                    circle_vertices(&body.position(), radius, COLORS[color.0], &mut vertices);
                }
            }
            for (position, shape) in debug_shapes.iter() {
                shape_vertices(position, shape, COLORS[0], &mut vertices);
            }

            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::vertex_buffer(),
                vertices.drain(..),
            ).expect("failed to create buffer");

            command_buffer_builder = command_buffer_builder
                .draw(
                    self.debug_pipeline.clone(),
                    screen_dynamic_state.clone(),
                    vec![vertex_buffer],
                    sets.clone(),
                    (),
                )
                .unwrap()
        }

        // Draw configuration menu
        let command_buffer_builder = {
            let mut imgui = world.write_resource::<::resource::ImGui>();
            // Values are wrong but it is for configuration purpose
            // so it doesn't matter
            let ui = imgui.frame(
                (dimensions[0], dimensions[1]),
                (dimensions[0], dimensions[1]),
                1.0 / 60.0,
            );

            ::config_menu::build(&ui, world);

            let ref_cell_cmd_builder = RefCell::new(Some(command_buffer_builder));
            ui.render::<_, ()>(|ui, drawlist| {
                let mut cmd_builder = ref_cell_cmd_builder.borrow_mut().take().unwrap();
                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::vertex_buffer(),
                    drawlist
                        .vtx_buffer
                        .iter()
                        .map(|vtx| ImGuiVertex::from(vtx.clone())),
                ).unwrap();

                let index_buffer = CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::index_buffer(),
                    drawlist.idx_buffer.iter().cloned(),
                ).unwrap();

                let (width, height) = ui.imgui().display_size();

                for _cmd in drawlist.cmd_buffer {
                    // dynamic scissor should be impl but don't care

                    cmd_builder = cmd_builder
                        .draw_indexed(
                            self.imgui_pipeline.clone(),
                            screen_dynamic_state.clone(),
                            vec![vertex_buffer.clone()],
                            index_buffer.clone(),
                            self.imgui_descriptor_set.clone(),
                            imgui_vs::ty::Dim {
                                width: width as f32,
                                height: height as f32,
                            },
                        )
                        .unwrap();
                }
                *ref_cell_cmd_builder.borrow_mut() = Some(cmd_builder);
                Ok(())
            }).unwrap();

            let command_buffer_builder = ref_cell_cmd_builder.borrow_mut().take().unwrap();
            command_buffer_builder
        };

        // TODO: Draw UI

        command_buffer_builder
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap()
    }

    pub fn draw(&mut self, world: &mut World, window: &Arc<Surface<::winit::Window>>) {
        self.future.as_mut().unwrap().cleanup_finished();

        // On X with Xmonad and intel HD graphics the acquire stay sometimes forever
        let timeout = Duration::from_secs(2);
        let mut next_image = swapchain::acquire_next_image(self.swapchain.clone(), Some(timeout));
        loop {
            match next_image {
                Err(AcquireError::OutOfDate) | Err(AcquireError::Timeout) => {
                    self.recreate(&window);
                    next_image =
                        swapchain::acquire_next_image(self.swapchain.clone(), Some(timeout));
                }
                _ => break,
            }
        }

        let (image_num, acquire_future) = next_image.unwrap();

        let command_buffer = self.build_command_buffer(image_num, world);

        let future = self.future
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.future = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                self.future = Some(Box::new(now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
                self.future = Some(Box::new(now(self.device.clone())) as Box<_>);
            }
        }
    }
}

const DIV: usize = 16;

lazy_static! {
    static ref DISK: Vec<::na::Point2<f32>> = (0..DIV + 1)
        .flat_map(|i| {
            let a1 = i as f32 * 2.0 * PI / DIV as f32;
            let a2 = (i + 1) as f32 * 2.0 * PI / DIV as f32;

            vec![
                ::na::Point2::new(a1.cos(), a1.sin()),
                ::na::Point2::new(a2.cos(), a2.sin()),
                ::na::Point2::new(0.0, 0.0),
            ]
        })
        .collect::<Vec<_>>();
}

lazy_static! {
    static ref CIRCLE: Vec<::na::Point2<f32>> = (0..DIV + 1)
        .flat_map(|i| {
            let outer_radius = 1.05;
            let inner_radius = 0.95;

            let a1 = i as f32 * 2.0 * PI / DIV as f32;
            let a2 = (i + 1) as f32 * 2.0 * PI / DIV as f32;

            let p1 = ::na::Point2::new(a1.cos(), a1.sin());
            let p2 = ::na::Point2::new(a2.cos(), a2.sin());

            let inner_p1 = p1 * inner_radius;
            let outer_p1 = p1 * outer_radius;

            let inner_p2 = p2 * inner_radius;
            let outer_p2 = p2 * outer_radius;

            vec![outer_p1, inner_p1, outer_p2, outer_p2, inner_p2, inner_p1]
        })
        .collect::<Vec<_>>();
}

lazy_static! {
    static ref COLORS: Vec<[f32; 4]> = vec![
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0],
    ];
}

fn circle_vertices(position: &::na::Isometry2<f32>, radius: f32, color: [f32; 4], vertices: &mut Vec<DebugVertex>) {
    vertices.extend(CIRCLE.iter().map(|p| *p * radius).map(|p| {
        let p = position * p;
        DebugVertex {
            position: [p[0], -p[1]],
            color,
        }
    }));
}
fn shape_vertices(position: &::na::Isometry2<f32>, shape: &ShapeHandle<f32>, color: [f32; 4], vertices: &mut Vec<DebugVertex>) {
    if let Some(ball) = shape.as_shape::<shape::Ball<f32>>() {
        vertices.extend(
            DISK.iter()
                .map(|p| *p * ball.radius())
                .map(|p| {
                    let p = position * p;
                    DebugVertex {
                        position: [p[0], -p[1]],
                        color,
                    }
                })
        );
    }
    if let Some(convex_polygon) = shape.as_shape::<shape::ConvexPolygon<f32>>() {
        let mut points_iter = convex_polygon.points().iter();
        let pivot = points_iter.next().unwrap();
        vertices.extend(
            points_iter
                .tuples()
                .flat_map(|(p1, p2)| vec![pivot, p1, p2])
                .map(|p| {
                    let p = position * *p;
                    DebugVertex {
                        position: [p[0], -p[1]],
                        color,
                    }
                })
                .collect::<Vec<_>>(),
        );
    }
    if let Some(segment) = shape.as_shape::<shape::Segment<f32>>() {
        let direction = segment.scaled_direction().normalize();
        let normal = ::na::Vector2::new(-direction[1], direction[0]);

        vertices.extend(
            [
                segment.a() + normal * DEBUG_SEGMENT_WIDTH / 2.0,
                segment.a() - normal * DEBUG_SEGMENT_WIDTH / 2.0,
                segment.b() - normal * DEBUG_SEGMENT_WIDTH / 2.0,
                segment.b() + normal * DEBUG_SEGMENT_WIDTH / 2.0,
                segment.b() - normal * DEBUG_SEGMENT_WIDTH / 2.0,
                segment.a() + normal * DEBUG_SEGMENT_WIDTH / 2.0,
            ].iter()
                .map(|p| {
                    let p = position * *p;
                    DebugVertex {
                        position: [p[0], -p[1]],
                        color,
                    }
                })
                .collect::<Vec<_>>(),
        );
    }
}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

layout(push_constant) uniform Info {
    float layer;
    float height;
    float width;
} info;

layout(set = 0, binding = 0) uniform View {
    mat4 view;
} view;

layout(set = 0, binding = 1) uniform World {
    mat4 world;
} world;

void main() {
    gl_Position = view.view * world.world * vec4(position[0]*info.width, position[1]*info.height, info.layer, 1.0);
    tex_coords = position + vec2(0.5);
}
"]
    struct _Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, tex_coords);
}
"]
    struct _Dummy;
}

mod debug_vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 v_color;

layout(set = 0, binding = 0) uniform View {
    mat4 view;
} view;

void main() {
    gl_Position = view.view * vec4(position, 0.1, 1.0);
    v_color = color;
}
"]
    struct _Dummy;
}

mod debug_fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450

layout(location = 0) in vec4 v_color;

layout(location = 0) out vec4 f_color;

void main() {
    f_color = v_color;
}
"]
    struct _Dummy;
}

mod imgui_vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(push_constant) uniform Dim {
    float width;
    float height;
} dim;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 col;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out vec4 f_color;

void main() {
    f_uv = uv;
    f_color = col / 255.0;

    mat4 matrix = mat4(
        vec4(2.0 / dim.width, 0.0, 0.0, 0.0),
        vec4(0.0, 2.0 / -dim.height, 0.0, 0.0),
        vec4(0.0, 0.0, -1.0, 0.0),
        vec4(-1.0, 1.0, 0.0, 1.0));

    gl_Position = matrix * vec4(pos.xy, 0, 1);
    gl_Position.y = - gl_Position.y;
}
"]
    struct _Dummy;
}

mod imgui_fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(location = 0) in vec2 f_uv;
layout(location = 1) in vec4 f_color;

layout(location = 0) out vec4 out_color;

void main() {
  out_color = f_color * texture(tex, f_uv.st);
}
"]
    struct _Dummy;
}

pub struct CustomRenderPassDesc {
    swapchain_image_format: Format,
}

unsafe impl RenderPassDesc for CustomRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        2
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<LayoutAttachmentDescription> {
        match id {
            // Colors
            0 => Some(LayoutAttachmentDescription {
                format: self.swapchain_image_format,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::Store,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            }),
            // Depth buffer
            1 => Some(LayoutAttachmentDescription {
                format: Format::D16Unorm,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::DontCare,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::DontCare,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        1
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<LayoutPassDescription> {
        match id {
            // draw
            0 => Some(LayoutPassDescription {
                color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: Some((1, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        0
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<LayoutPassDependencyDescription> {
        match id {
            _ => None,
        }
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for CustomRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}
