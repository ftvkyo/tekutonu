use std::sync::Arc;

use vulkano::{
    buffer::TypedBufferAccess,
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        RenderPassBeginInfo,
        SubpassContents,
    },
    device::{Device, Queue},
    instance::Instance,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
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
    sync::{self, FlushError, GpuFuture},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use super::{framebuffer::make_framebuffers, make_vertex_buffer};


pub struct GameRenderer {
    device: Arc<Device>,
    queues: Vec<Arc<Queue>>,
    event_loop: EventLoop<()>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,
}

impl GameRenderer {
    pub fn new(instance: Arc<Instance>) -> Self {
        use vulkano_win::VkSurfaceBuild;
        use winit::window::WindowBuilder;

        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
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
        // resized Otherwise we would have to recreate the whole pipeline.
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
            event_loop,
            surface,
            swapchain,
            render_pass,
            pipeline,
            viewport,
            framebuffers,
        }
    }

    pub fn render(mut self) -> () {
        // Describe our triangle
        let vertex_buffer = make_vertex_buffer(self.device.clone());

        let mut should_recreate_swapchain = false;

        // Store the submission of the previous frame to avoid blocking on GpuFutures to
        // wait.
        let mut previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.event_loop.run(move |event, _, control_flow| {
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

                    // It is important to call this function from time to time, otherwise resources
                    // will keep accumulating and you will eventually reach an out
                    // of memory error. Calling this function polls various fences
                    // in order to determine what the GPU has already processed, and
                    // frees the resources that are no longer needed.
                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    // Whenever the window resizes we need to recreate everything dependent on the
                    // window size. In this example that includes the swapchain, the
                    // framebuffers and the dynamic state viewport.
                    if should_recreate_swapchain {
                        // Use the new dimensions of the window.

                        let (new_swapchain, new_images) =
                            match self.swapchain.recreate(SwapchainCreateInfo {
                                image_extent: dimensions.into(),
                                ..self.swapchain.create_info()
                            }) {
                                Ok(r) => r,
                                // This error tends to happen when the user is manually resizing the
                                // window. Simply restarting the loop is the
                                // easiest way to fix this issue.
                                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                                    return
                                },
                                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                            };

                        self.swapchain = new_swapchain;
                        // Because framebuffers contains an Arc on the old swapchain, we need to
                        // recreate framebuffers as well.
                        self.framebuffers = make_framebuffers(
                            &new_images,
                            self.render_pass.clone(),
                            &mut self.viewport,
                        );
                        should_recreate_swapchain = false;
                    }

                    // Before we can draw on the output, we have to *acquire* an image from the
                    // swapchain. If no image is available (which happens if you
                    // submit draw commands too quickly), then the function will
                    // block. This operation returns the index of the image that we
                    // are allowed to draw upon.
                    //
                    // This function can block if no image is available. The parameter is an
                    // optional timeout after which the function call will return an
                    // error.
                    let (image_num, suboptimal, acquire_future) =
                        match acquire_next_image(self.swapchain.clone(), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                should_recreate_swapchain = true;
                                return;
                            },
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };

                    // acquire_next_image can be successful, but suboptimal. This means that the
                    // swapchain image will still work, but it may not display
                    // correctly. With some drivers this can be when the window
                    // resizes, but it may not cause the swapchain to become out of date.
                    if suboptimal {
                        should_recreate_swapchain = true;
                    }

                    // In order to draw, we have to build a *command buffer*. The command buffer
                    // object holds the list of commands that are going to be
                    // executed.
                    //
                    // Building a command buffer is an expensive operation (usually a few hundred
                    // microseconds), but it is known to be a hot path in the driver and is expected
                    // to be optimized.
                    //
                    // Note that we have to pass a queue family when we create the command buffer.
                    // The command buffer will only be executable on that given
                    // queue family.
                    let mut builder = AutoCommandBufferBuilder::primary(
                        self.device.clone(),
                        self.queues[0].queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();

                    builder
                        // Before we can draw, we have to *enter a render pass*.
                        .begin_render_pass(
                            RenderPassBeginInfo {
                                // A list of values to clear the attachments with. This list
                                // contains one item for each
                                // attachment in the render pass. In this case,
                                // there is only one attachment, and we clear it with a blue color.
                                //
                                // Only attachments that have `LoadOp::Clear` are provided with
                                // clear values, any others should
                                // use `ClearValue::None` as the clear value.
                                clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                                ..RenderPassBeginInfo::framebuffer(
                                    self.framebuffers[image_num].clone(),
                                )
                            },
                            // The contents of the first (and only) subpass. This can be either
                            // `Inline` or `SecondaryCommandBuffers`. The latter is a bit more
                            // advanced and is not covered here.
                            SubpassContents::Inline,
                        )
                        .unwrap()
                        // We are now inside the first subpass of the render pass. We add a draw
                        // command.
                        //
                        // The last two parameters contain the list of resources to pass to the
                        // shaders. Since we used an `EmptyPipeline` object,
                        // the objects have to be `()`.
                        .set_viewport(0, [self.viewport.clone()])
                        .bind_pipeline_graphics(self.pipeline.clone())
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .draw(vertex_buffer.len() as u32, 1, 0, 0)
                        .unwrap()
                        // We leave the render pass. Note that if we had multiple
                        // subpasses we could have called `next_subpass` to jump to the next
                        // subpass.
                        .end_render_pass()
                        .unwrap();

                    // Finish building the command buffer by calling `build`.
                    let command_buffer = builder.build().unwrap();

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(self.queues[0].clone(), command_buffer)
                        .unwrap()
                        // The color output is now expected to contain our triangle. But in order to
                        // show it on the screen, we have to *present* the image by
                        // calling `present`.
                        //
                        // This function does not actually present the image immediately. Instead it
                        // submits a present command at the end of the queue. This
                        // means that it will only be presented once the GPU has
                        // finished executing the command buffer that draws the triangle.
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
