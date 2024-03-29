use std::sync::Arc;

use vulkano::{
    device::Device,
    image::{ImageUsage, SwapchainImage},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use winit::window::Window;

pub fn make_swapchain_and_images(
    device: Arc<Device>,
    window: Arc<Window>,
    surface: Arc<Surface>,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    // We will only be allowed to request capabilities that are supported by the
    // surface
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    // Internal format of the image buffers of the swapchain
    let surface_format = Some(
        device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );

    // Size of image buffers of the swapchain
    let swap_extent = match surface_capabilities.current_extent {
        Some(extent) => extent,
        None => window.inner_size().into(),
    };

    Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            image_format: surface_format,
            image_extent: swap_extent,

            // Can never create less than what surface allows.
            min_image_count: surface_capabilities.min_image_count,

            // We can only use images in the ways we have requested.
            image_usage: ImageUsage {
                color_attachment: true,
                ..ImageUsage::empty()
            },

            /*
            // Сompositing with other windows using the alpha channel
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap(),
            */
            ..Default::default()
        },
    )
    .unwrap()
}
