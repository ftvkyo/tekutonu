use std::sync::Arc;

use vulkano::{
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, SwapchainImage},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

pub fn make_framebuffers(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    allocator: Arc<StandardMemoryAllocator>,
) -> Vec<Arc<Framebuffer>> {
    // This method is called once during initialization, then again whenever the
    // window is resized
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(&allocator, dimensions, Format::D32_SFLOAT).unwrap(),
    )
    .unwrap();

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
