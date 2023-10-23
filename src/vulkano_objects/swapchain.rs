use std::sync::Arc;

use vulkano::{
    device::Device,
    image::SwapchainImage,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError},
};
use winit::window::Window;



pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    let usage = capabilities.supported_usage_flags;
    let alpha = capabilities
        .supported_composite_alpha
        .into_iter()
        .next()
        .unwrap();

    let image_format = Some(
        device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );

    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
    let image_extent: [u32; 2] = window.inner_size().into();

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: capabilities.min_image_count,
            image_format,
            image_extent,
            image_usage: usage,
            composite_alpha: alpha,
            ..Default::default()
        },
    )
    .unwrap()
}

pub fn swapchain_from(surface: Arc<Surface>, swapchain: Arc<Swapchain>) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
    let image_extent: [u32; 2] = window.inner_size().into();

    let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
        image_extent,
        ..swapchain.create_info()
    }) {
        Ok(r) => r,
        Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return (swapchain, vec![]),
        Err(e) => panic!("Failed to create swapchain: {:?}", e),
    };
    (new_swapchain, new_images)
}