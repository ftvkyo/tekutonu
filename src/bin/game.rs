use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        RenderPassBeginInfo,
        SubpassContents,
    },
    device::{
        physical::PhysicalDeviceType,
        Device,
        DeviceCreateInfo,
        DeviceExtensions,
        Queue,
        QueueCreateInfo,
    },
    image::{view::ImageView, ImageAccess, ImageUsage, SwapchainImage},
    impl_vertex,
    instance::{Instance, InstanceCreateInfo},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
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
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn make_instance() -> Arc<Instance> {
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = vulkano_win::required_extensions(&library);

    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            // Enable enumerating devices that use non-conformant vulkan implementations.
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap()
}

fn choose_device_and_queue(
    instance: Arc<Instance>,
    surface: Arc<Surface<Window>>,
) -> (Arc<Device>, Arc<Queue>) {
    // TODO: check out what other extensions are there
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|pd| pd.supported_extensions().contains(&device_extensions))
        .filter_map(|pd| {
            // TODO: maybe use separate queues for data transfer and graphics operations.
            pd.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, queue)| {
                    // Graphics operations
                    queue.queue_flags.graphics
                    // Supports presenting to out surface
                    && pd.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (pd, i as u32))
        })
        .min_by_key(|(pd, _)| match pd.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("No suitable physical device found");

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    // Logical device
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    (device, queues.next().unwrap())
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec2 position;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            layout(location = 0) out vec4 f_color;
                void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "
    }
}

fn create_swapchain_and_images(device: Arc<Device>, surface: Arc<Surface<Window>>) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    // We will only be allowed to request capabilities that are supported by the
    // surface
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    // Internal format of the image buffers of the swapchain
    let image_format = Some(
        device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );

    // Size of image buffers of the swapchain
    let image_extent = match surface_capabilities.current_extent {
        Some(extent) => extent,
        None => surface.window().inner_size().into(),
    };

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            image_format,
            image_extent,

            // Can never create less than what surface allows.
            min_image_count: surface_capabilities.min_image_count,

            // We can only use images in the ways we have requested.
            image_usage: ImageUsage {
                color_attachment: true,
                ..ImageUsage::empty()
            },

            // TODO: read more. maybe this means transparent window and compositing with other
            // windows?
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap(),

            ..Default::default()
        },
    )
    .unwrap()
}

fn main() {
    let instance = make_instance();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let (device, queue) = choose_device_and_queue(instance.clone(), surface.clone());

    // Allocating color (image) buffers through creating a swapchain.
    let (mut swapchain, images) = create_swapchain_and_images(device.clone(), surface.clone());

    // How we are going to give data to the device
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
    struct Vertex {
        position: [f32; 2],
    }
    impl_vertex!(Vertex, position);

    // Describe our triangle
    let vertices = [
        Vertex {
            position: [-0.5, -0.25],
        },
        Vertex {
            position: [0.0, 0.5],
        },
        Vertex {
            position: [0.25, -0.1],
        },
    ];
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        vertices,
    )
    .unwrap();

    // Creating shaders
    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    // Describe where the output of the graphics pipeline will go by creating a
    // RenderPass.
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        // TODO: read about attachments
        attachments: {
            // Making the only attachment with a custom name `color`
            color: {
                // Clear the content of the attachment at the start of drawing.
                load: Clear,
                // Actually store the output of the draw in the image...
                store: Store,
                format: swapchain.image_format(),
                // Don't do multisampling, we don't want antialiasing (yet)
                samples: 1,
            }
        },
        pass: {
            // Use the attachment named `color`
            color: [color],
            // No depth-stencil attachment
            // TODO: read about it
            depth_stencil: {}
        }
    )
    .unwrap();

    // Specify what we want the device to do
    let pipeline = GraphicsPipeline::start()
        // Which subpass of which render pass this pipeline is going to be used in.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // How the vertices are laid out.
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .build(device.clone())
        .unwrap();

    // Dynamic viewports allow us to recreate just the viewport when the window is
    // resized Otherwise we would have to recreate the whole pipeline.
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    // render_pass only specifies the layout of framebuffers, we need to actually
    // create them. We should create a separate framebuffer for every image.
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);

    // End of initialization.

    let mut should_recreate_swapchain = false;

    // Store the submission of the previous frame to avoid blocking on GpuFutures to
    // wait.
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

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
                let dimensions = surface.window().inner_size();
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
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: dimensions.into(),
                            ..swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            // This error tends to happen when the user is manually resizing the
                            // window. Simply restarting the loop is the
                            // easiest way to fix this issue.
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    swapchain = new_swapchain;
                    // Because framebuffers contains an Arc on the old swapchain, we need to
                    // recreate framebuffers as well.
                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
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
                    match acquire_next_image(swapchain.clone(), None) {
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
                    device.clone(),
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    // Before we can draw, we have to *enter a render pass*.
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            // A list of values to clear the attachments with. This list contains
                            // one item for each attachment in the render pass. In this case,
                            // there is only one attachment, and we clear it with a blue color.
                            //
                            // Only attachments that have `LoadOp::Clear` are provided with clear
                            // values, any others should use `ClearValue::None` as the clear value.
                            clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffers[image_num].clone())
                        },
                        // The contents of the first (and only) subpass. This can be either
                        // `Inline` or `SecondaryCommandBuffers`. The latter is a bit more advanced
                        // and is not covered here.
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    // We are now inside the first subpass of the render pass. We add a draw
                    // command.
                    //
                    // The last two parameters contain the list of resources to pass to the shaders.
                    // Since we used an `EmptyPipeline` object, the objects have to be `()`.
                    .set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    // We leave the render pass. Note that if we had multiple
                    // subpasses we could have called `next_subpass` to jump to the next subpass.
                    .end_render_pass()
                    .unwrap();

                // Finish building the command buffer by calling `build`.
                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
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
                        queue.clone(),
                        PresentInfo {
                            index: image_num,
                            ..PresentInfo::swapchain(swapchain.clone())
                        },
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    },
                    Err(FlushError::OutOfDate) => {
                        should_recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    },
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    },
                }
            },
            _ => (),
        }
    });
}

/// This method is called once during initialization, then again whenever the
/// window is resized
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
