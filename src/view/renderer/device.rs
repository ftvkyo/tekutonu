use std::sync::Arc;

use vulkano::{
    device::{
        physical::PhysicalDeviceType,
        Device,
        DeviceCreateInfo,
        DeviceExtensions,
        Queue,
        QueueCreateInfo,
    },
    instance::Instance,
    swapchain::Surface,
};


pub fn choose_device_and_queue(
    instance: Arc<Instance>,
    surface: Arc<Surface>,
) -> (Arc<Device>, Vec<Arc<Queue>>) {
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
    let (device, queues) = Device::new(
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

    (device, queues.collect())
}
