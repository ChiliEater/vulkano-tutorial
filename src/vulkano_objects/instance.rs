use std::sync::Arc;

use vulkano::{instance::{Instance, InstanceCreateInfo}, VulkanLibrary, Version};

pub fn new() -> Arc<Instance> {
    let library = VulkanLibrary::new().unwrap();
    let extensions = vulkano_win::required_extensions(&library);

    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: extensions,
            enumerate_portability: true, // MoltenVK compat
            max_api_version: Some(Version::V1_1),
            ..Default::default()
        },
    )
    .unwrap()
}