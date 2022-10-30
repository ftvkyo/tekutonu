use std::{sync::Arc, time::Instant};

use cgmath::{Matrix3, Rad, Point3, Vector3, Matrix4};
use vulkano::{
    buffer::{CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        PrimaryAutoCommandBuffer,
        RenderPassBeginInfo,
        SubpassContents,
    },
    device::{Device, Queue},
    instance::Instance,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    swapchain::{
        acquire_next_image,
        AcquireError,
        PresentInfo,
        Surface,
        Swapchain,
        SwapchainCreateInfo,
        SwapchainCreationError,
    },
    sync::{self, FlushError, GpuFuture}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};

use super::Vertex;


pub struct GameRenderer {
    device: Arc<Device>,
    queues: Vec<Arc<Queue>>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,
}

impl GameRenderer {
    pub fn new(instance: Arc<Instance>, event_loop: &EventLoop<()>) -> Self {
        use vulkano_win::VkSurfaceBuild;
        use winit::window::WindowBuilder;

        let surface = WindowBuilder::new()
            .build_vk_surface(event_loop, instance.clone())
            .unwrap();

        let (device, queues) = super::device::choose_device_and_queue(instance, surface.clone());

        // Allocating color (image) buffers through creating a swapchain.
        let (swapchain, images) =
            super::swapchain::make_swapchain_and_images(device.clone(), surface.clone());

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
        // However, not using a dynamic viewport could allow the driver to optimize some things at the cost of slower resizes.
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
            surface,
            swapchain,
            render_pass,
            pipeline,
            viewport,
            framebuffers,
        }
    }

    fn build_command_buffer(
        &mut self,
        image_num: usize,
        descriptor_set: Arc<PersistentDescriptorSet>,
        vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
        index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,
    ) -> PrimaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
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
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
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
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .bind_index_buffer(index_buffer.clone())
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        builder.build().unwrap()
    }

    fn recreate_swapchain(
        &mut self,
        dimensions: PhysicalSize<u32>,
    ) -> Result<(Arc<Swapchain<Window>>, Vec<Arc<Framebuffer>>), ()> {
        let (new_swapchain, new_images) =
        match self.swapchain.recreate(SwapchainCreateInfo {
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

    pub fn render(mut self, event_loop: EventLoop<()>) {
        // Describe our square
        let vertex_buffer = super::make_vertex_buffer(self.device.clone());
        let index_buffer = super::make_index_buffer(self.device.clone());

        // Describe world rotation and camera position
        let uniform_buffer = super::make_uniforms_buffer(self.device.clone());

        let rotation_start = Instant::now();

        let mut should_recreate_swapchain = false;

        // Store the submission of the previous frame to avoid blocking on GpuFutures to
        // wait.
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    should_recreate_swapchain = true;
                },
                Event::RedrawEventsCleared => {
                    // Do not draw frame when screen dimensions are zero.
                    // On Windows, this can occur from minimizing the application.
                    let dimensions = self.surface.window().inner_size();
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
                            }
                        }
                        should_recreate_swapchain = false;
                    }

                    let uniform_buffer_subbuffer = {
                        let elapsed = rotation_start.elapsed();
                        let rotation =
                            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
                        let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

                        let aspect_ratio =
                            self.swapchain.image_extent()[0] as f32 / self.swapchain.image_extent()[1] as f32;
                        let proj = cgmath::perspective(
                            Rad(std::f32::consts::FRAC_PI_2),
                            aspect_ratio,
                            0.01,
                            100.0,
                        );
                        let view = Matrix4::look_at_rh(
                            Point3::new(0.3, 0.3, 1.0),
                            Point3::new(0.0, 0.0, 0.0),
                            Vector3::new(0.0, -1.0, 0.0),
                        );
                        let scale = Matrix4::from_scale(0.5);

                        let uniform_data = super::shaders::vs::ty::Data {
                            world: Matrix4::from(rotation).into(),
                            view: (view * scale).into(),
                            proj: proj.into(),
                        };

                        uniform_buffer.from_data(uniform_data).unwrap()
                    };

                    let pipeline_layout = self.pipeline.layout().set_layouts().get(0).unwrap();
                    let descriptor_set = PersistentDescriptorSet::new(
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

                    let command_buffer =
                        self.build_command_buffer(image_num, descriptor_set.clone(), vertex_buffer.clone(), index_buffer.clone());

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(self.queues[0].clone(), command_buffer)
                        .unwrap()
                        // Submit a present command at the end of the queue.
                        .then_swapchain_present(
                            self.queues[0].clone(),
                            PresentInfo {
                                index: image_num,
                                ..PresentInfo::swapchain(self.swapchain.clone())
                            },
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
