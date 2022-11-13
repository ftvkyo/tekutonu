use std::sync::Arc;

use tracing::instrument;
use vulkano::instance::Instance;

#[instrument]
pub fn make_instance() -> Arc<Instance> {
    use vulkano::{instance::InstanceCreateInfo, VulkanLibrary};

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
