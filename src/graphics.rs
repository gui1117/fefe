use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{self, Swapchain};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode,
                       UnnormalizedSamplerAddressMode};
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, ImmutableImage, SwapchainImage};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer,
                      ImmutableBuffer};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, LayoutAttachmentDescription,
                           LayoutPassDependencyDescription, LayoutPassDescription, LoadOp,
                           RenderPass, RenderPassAbstract, RenderPassDesc,
                           RenderPassDescClearValues, StoreOp, Subpass};
use vulkano::pipeline::{ComputePipeline, GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::descriptor::descriptor_set::{DescriptorSet, FixedSizeDescriptorSetsPool,
                                          PersistentDescriptorSet, PersistentDescriptorSetBuf,
                                          PersistentDescriptorSetImg,
                                          PersistentDescriptorSetSampler};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::instance::PhysicalDevice;
use vulkano::sync::{now, GpuFuture};
use vulkano::image::ImageLayout;
use vulkano::format::{self, ClearValue, Format};
use vulkano::sync::{AccessFlagBits, PipelineStages};
use vulkano;

use std::ops;
use std::sync::Arc;
use std::fs::File;
use std::time::Duration;

// TODO: only a bool for whereas draw the cursor or not

pub struct Graphics<'a> {
    physical: PhysicalDevice<'a>,
    queue: Arc<Queue>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPassAbstract + Sync + Send>,
    pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    animation_images: Vec<Arc<DescriptorSet>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Sync + Send>>,
    view_buffer_pool: CpuBufferPool<vs::ty::View>,
    world_buffer_pool: CpuBufferPool<vs::ty::World>,
    future: Option<Box<GpuFuture>>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

impl<'a> Graphics<'a> {
    pub fn new(window: &'a ::vulkano_win::Window) -> Graphics<'a> {
        let physical = PhysicalDevice::enumerate(&window.surface().instance())
            .next()
            .expect("no device available");

        let queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && q.supports_compute()
                    && window.surface().is_supported(q).unwrap_or(false)
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
                .surface()
                .capabilities(physical)
                .expect("failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let format = caps.supported_formats[0].0;
            let image_usage = ImageUsage {
                // sampled: true,
                color_attachment: true,
                ..ImageUsage::none()
            };

            Swapchain::new(
                device.clone(),
                window.surface().clone(),
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
                [0.5, -0.5],
                [-0.5, 0.5],
            ].iter()
                .cloned()
                .map(|position| Vertex { position }),
            BufferUsage::vertex_buffer(),
            queue.clone(),
        ).expect("failed to create buffer");
        future = Box::new(future.join(vertex_buffer_fut)) as Box<_>;

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let mut animation_images = vec![];

        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            // TODO: What values here
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

            let image_descriptor_set = Arc::new(
                PersistentDescriptorSet::start(pipeline.clone(), 2)
                    .add_sampled_image(image.clone(), sampler.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<_>;

            animation_images.push(image_descriptor_set);
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
            vertex_buffer,
            animation_images,
            framebuffers,
            view_buffer_pool,
            world_buffer_pool,
            physical,
        }
    }

    fn recreate(&mut self, window: &::vulkano_win::Window) {
        let recreate;
        loop {
            // TODO: Sleep and max number of try
            let dimensions = window
                .surface()
                .capabilities(self.physical)
                .expect("failed to get surface capabilities")
                .current_extent
                .unwrap_or([1024, 768]);
            match self.swapchain.recreate_with_dimension(dimensions) {
                Err(::vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => (),
                r @ _ => {
                    recreate = Some(r);
                    break;
                }
            }
        }

        let (swapchain, images) = recreate.unwrap().unwrap();
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
        world: &mut ::specs::World,
    ) -> AutoCommandBuffer<StandardCommandPoolAlloc> {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        ).unwrap()
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                false,
                vec![[0.0, 0.0, 1.0, 1.0].into(), 1.0.into()],
            )
            .unwrap();

        // TODO view_set
        let mut images = world.write_resource::<::resource::AnimationImages>();
        for image in images.drain(..) {
            // TODO world_set
            command_buffer_build = command_buffer_builder.draw(
                self.pipeline.clone(),
                DynamicState::none(),
                self.vertex_buffers.clone(),
                (view_set.clone(), static_draw.set.clone()),
                vs::ty::Layer { layer: image.layer },
            )
        }

        command_buffer_builder
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap()
    }

    pub fn draw(&mut self, world: &mut ::specs::World, window: &::vulkano_win::Window) {
        self.future.as_mut().unwrap().cleanup_finished();

        // On X with Xmonad and intel HD graphics the acquire stay sometimes forever
        let timeout = Duration::from_secs(2);
        let mut next_image = swapchain::acquire_next_image(self.swapchain.clone(), Some(timeout));
        loop {
            match next_image {
                Err(vulkano::swapchain::AcquireError::OutOfDate)
                | Err(vulkano::swapchain::AcquireError::Timeout) => {
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
            .then_signal_fence_and_flush()
            .unwrap();

        self.future = Some(Box::new(future) as Box<_>);
    }
}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

layout(push_constant) uniform Layer {
    float layer;
} layer;

layout(set = 0, binding = 0) uniform View {
    mat4 view;
} view;

layout(set = 1, binding = 0) uniform World {
    mat4 world;
} world;

void main() {
    gl_Position = view.view * world.world * vec4(position, layer.layer, 1.0);
    // https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
    gl_Position.y = - gl_Position.y;
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

layout(set = 2, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, tex_coords);
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
