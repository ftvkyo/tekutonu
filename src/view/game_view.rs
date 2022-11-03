use std::{f32::consts::FRAC_PI_2, sync::Arc};

use cgmath::{InnerSpace, Matrix4, One, Rad, Vector3};
use tracing::instrument;
use vulkano::{
    buffer::{
        cpu_pool::CpuBufferPoolSubbuffer,
        CpuAccessibleBuffer,
        CpuBufferPool,
        TypedBufferAccess,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        PrimaryAutoCommandBuffer,
        RenderPassBeginInfo,
        SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        PersistentDescriptorSet,
        WriteDescriptorSet,
    },
    device::{Device, Queue},
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    swapchain::{
        acquire_next_image,
        AcquireError,
        Swapchain,
        SwapchainCreateInfo,
        SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
};
use vulkano_win::create_surface_from_winit;
use winit::{
    dpi::{PhysicalSize, Size},
    error::ExternalError,
    event::{DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Window},
};

use super::Vertex;
use crate::{
    controller::GameInput,
    model::{Camera, GameModel},
};


pub struct GameView {
    device: Arc<Device>,
    queues: Vec<Arc<Queue>>,
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,

    event_loop: Option<EventLoop<()>>,
}

impl GameView {
    #[instrument(skip_all)]
    pub fn new(vk: Arc<Instance>, event_loop: EventLoop<()>) -> Self {
        let window_builder = winit::window::WindowBuilder::new()
            .with_resizable(false)
            .with_inner_size(Size::Physical(PhysicalSize {
                width: 1920,
                height: 1080,
            }))
            // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .with_title("tekutonu");

        let window = Arc::new(window_builder.build(&event_loop).unwrap());
        let surface = create_surface_from_winit(window.clone(), vk.clone()).unwrap();

        let (device, queues) = super::device::choose_device_and_queue(vk, surface.clone());

        // Allocating color (image) buffers through creating a swapchain.
        let (swapchain, images) =
            super::swapchain::make_swapchain_and_images(device.clone(), window.clone(), surface);

        // Describe where the output of the graphics pipeline will go by creating a
        // RenderPass.
        let render_pass = super::render_pass::make_render_pass(device.clone(), swapchain.clone());

        // Loading shaders
        let vs = super::shaders::vs::load(device.clone()).unwrap();
        let fs = super::shaders::fs::load(device.clone()).unwrap();

        // Specify what we want the device to do
        let pipeline = super::pipeline::make_pipeline(device.clone(), render_pass.clone(), vs, fs);

        // Dynamic viewports allow us to recreate just the viewport when the window is
        // resized.
        // Otherwise we would have to recreate the whole pipeline.
        // However, not using a dynamic viewport could allow the driver to optimize some
        // things at the cost of slower resizes.
        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        // render_pass only specifies the layout of framebuffers, we need to actually
        // create them. We should create a separate framebuffer for every image.
        let framebuffers =
            super::framebuffer::make_framebuffers(&images, render_pass.clone(), &mut viewport);

        // End of initialization.

        Self {
            device,
            queues,
            window,
            swapchain,
            render_pass,
            pipeline,
            viewport,
            framebuffers,
            event_loop: Some(event_loop),
        }
    }

    fn build_command_buffer(
        &mut self,
        image_num: usize,
        allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set: Arc<PersistentDescriptorSet>,
        vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
        index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,
    ) -> PrimaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary(
            &allocator,
            // Can only use the command buffer with this queue
            self.queues[0].queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    // One item for each attachment in the render pass that have `LoadOp::Clear`
                    // (otherwise None)
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_num].clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .bind_vertex_buffers(0, vertex_buffer)
            .bind_index_buffer(index_buffer.clone())
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        builder.build().unwrap()
    }

    #[instrument(skip_all)]
    fn recreate_swapchain(
        &mut self,
        dimensions: PhysicalSize<u32>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<Framebuffer>>), ()> {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions.into(),
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            // Likely user resizing the window, just retry.
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                return Err(());
            },
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };

        // Framebuffers depend on the images
        let new_framebuffers = super::framebuffer::make_framebuffers(
            &new_images,
            self.render_pass.clone(),
            &mut self.viewport,
        );

        Ok((new_swapchain, new_framebuffers))
    }

    fn create_uniform_subbuffer(
        &mut self,
        uniform_buffer_pool: CpuBufferPool<super::shaders::vs::ty::Data>,
        camera: &Camera,
    ) -> Arc<CpuBufferPoolSubbuffer<super::shaders::vs::ty::Data>> {
        let position = camera.position.map(|v| v as f32);

        let direction_y = f32::sin(camera.pitch.0);
        // Scale horizontal coordinates down to how much they matter based on the pitch
        let direction_x = f32::cos(camera.yaw.0) * f32::cos(camera.pitch.0);
        let direction_z = f32::sin(camera.yaw.0) * f32::cos(camera.pitch.0);
        let direction = Vector3::new(-direction_x, -direction_y, -direction_z).normalize();

        let aspect_ratio =
            self.swapchain.image_extent()[0] as f32 / self.swapchain.image_extent()[1] as f32;

        // Perspective projection matrix
        let proj = cgmath::perspective(
            // 90 degrees
            Rad(FRAC_PI_2),
            aspect_ratio,
            0.01,
            100.0,
        );

        let view =
            Matrix4::look_at_rh(position, position + direction, Vector3::new(0.0, -1.0, 0.0));

        let scale = Matrix4::from_scale(0.5);

        let uniform_data = super::shaders::vs::ty::Data {
            world: Matrix4::one().into(),
            view: (view * scale).into(),
            proj: proj.into(),
        };

        uniform_buffer_pool.from_data(uniform_data).unwrap()
    }

    #[instrument(skip_all)]
    pub fn set_cursor_locked(&self, locked: bool) -> Result<(), ExternalError> {
        let grab = if locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        let res = self.window.set_cursor_grab(grab);

        if res.is_err() && locked {
            self.window.set_cursor_grab(CursorGrabMode::Confined)
        } else {
            res
        }
    }

    #[instrument(skip_all)]
    pub fn set_cursor_hidden(&self, hidden: bool) {
        self.window.set_cursor_visible(!hidden)
    }

    #[instrument(skip_all)]
    pub fn run(mut self, mut game: GameModel, input: GameInput) {
        let event_loop = self.event_loop.take().unwrap();

        let allocator_memory = Arc::new(StandardMemoryAllocator::new_default(self.device.clone()));
        let allocator_descriptor_set =
            Arc::new(StandardDescriptorSetAllocator::new(self.device.clone()));
        let allocator_command = Arc::new(StandardCommandBufferAllocator::new(
            self.device.clone(),
            Default::default(),
        ));

        let mut modifiers = ModifiersState::default();

        // Describe our square
        let vertex_buffer = super::make_vertex_buffer(allocator_memory.clone());
        let index_buffer = super::make_index_buffer(allocator_memory.clone());

        // Describe world rotation and camera position
        let uniform_buffer_pool = super::make_uniforms_buffer(allocator_memory);

        let mut should_recreate_swapchain = false;

        // Store the submission of the previous frame to avoid blocking on GpuFutures to
        // wait.
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(_) => should_recreate_swapchain = true,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    } => input.process_keyboard_input(&self, key, control_flow),
                    WindowEvent::ModifiersChanged(m) => modifiers = m,
                    _ => (),
                },
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta, .. },
                    ..
                } => {
                    let effect = input.mouse_movement(delta);
                    game.apply_effect(effect);
                },
                Event::RedrawEventsCleared => {
                    // Do not draw frame when screen dimensions are zero.
                    // On Windows, this can occur from minimizing the application.
                    let dimensions = self.window.inner_size();
                    if dimensions.width == 0 || dimensions.height == 0 {
                        return;
                    }

                    // Periodic garbage collection.
                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    // Whenever the window resizes we need to recreate everything dependent on the
                    // window size.
                    if should_recreate_swapchain {
                        match self.recreate_swapchain(dimensions) {
                            Ok((swapchain, framebuffers)) => {
                                self.swapchain = swapchain;
                                self.framebuffers = framebuffers;
                            },
                            _ => {
                                return;
                            },
                        }
                        should_recreate_swapchain = false;
                    }

                    let uniform_buffer_subbuffer =
                        self.create_uniform_subbuffer(uniform_buffer_pool.clone(), &game.camera);

                    let pipeline_layout = self.pipeline.layout().set_layouts().get(0).unwrap();
                    let descriptor_set = PersistentDescriptorSet::new(
                        &allocator_descriptor_set,
                        pipeline_layout.clone(),
                        [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
                    )
                    .unwrap();

                    // Acquire image from the swapchain for drawing. Wait if no image is yet
                    // available.
                    let (image_num, suboptimal, acquire_future) =
                        match acquire_next_image(self.swapchain.clone(), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                should_recreate_swapchain = true;
                                return;
                            },
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };

                    // May happen when window is being resized.
                    // We can still render, but we should recreate it when we have a chance.
                    if suboptimal {
                        should_recreate_swapchain = true;
                    }

                    let command_buffer = self.build_command_buffer(
                        image_num as usize,
                        allocator_command.clone(),
                        descriptor_set,
                        vertex_buffer.clone(),
                        index_buffer.clone(),
                    );

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(self.queues[0].clone(), command_buffer)
                        .unwrap()
                        // Submit a present command at the end of the queue.
                        .then_swapchain_present(
                            self.queues[0].clone(),
                            SwapchainPresentInfo::swapchain_image_index(
                                self.swapchain.clone(),
                                image_num,
                            ),
                        )
                        .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => {
                            previous_frame_end = Some(future.boxed());
                        },
                        Err(FlushError::OutOfDate) => {
                            should_recreate_swapchain = true;
                            previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                        },
                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);
                            previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                        },
                    }
                },
                _ => (),
            }
        });
    }
}
