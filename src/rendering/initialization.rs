use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
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
    shader::ShaderModule,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    VulkanLibrary,
};
use winit::window::Window;

pub fn make_instance() -> Arc<Instance> {
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

pub fn choose_device_and_queue(
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

pub fn create_swapchain_and_images(
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
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
        device,
        surface,
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

// How we are going to give data to the device
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

pub fn make_vertex_buffer(device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
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

    CpuAccessibleBuffer::from_iter(
        device,
        BufferUsage {
            vertex_buffer: true,
            ..BufferUsage::empty()
        },
        false,
        vertices,
    )
    .unwrap()
}

pub fn make_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
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
    .unwrap()
}

pub fn make_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        // Which subpass of which render pass this pipeline is going to be used in.
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        // How the vertices are laid out.
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .build(device)
        .unwrap()
}

/// This method is called once during initialization, then again whenever the
/// window is resized
pub fn window_size_dependent_setup(
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
