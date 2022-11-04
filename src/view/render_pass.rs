use std::sync::Arc;

use vulkano::{device::Device, render_pass::RenderPass, swapchain::Swapchain};

pub fn make_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
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
