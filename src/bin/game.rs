use vulkano::{VulkanLibrary, device::{DeviceExtensions, physical::PhysicalDeviceType}, instance::{Instance, InstanceCreateInfo}};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = vulkano_win::required_extensions(&library);

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            // Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    // TODO: check out what other extensions are there

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|pd| {
            pd.supported_extensions().contains(&device_extensions)
        })
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
        .min_by_key(|(pd, _)| {
            match pd.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        })
        .expect("No suitable physical device found");

    // Some little debug infos.
    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );
}
