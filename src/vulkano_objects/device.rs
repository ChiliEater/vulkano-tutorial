use std::sync::Arc;

use vulkano::{device::{physical::{PhysicalDevice, PhysicalDeviceType}, DeviceExtensions, QueueFlags}, instance::Instance, swapchain::Surface};

pub fn query_device(instance: Arc<Instance>, surface: Arc<Surface>) -> (Arc<PhysicalDevice>, u32) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    // pick first queue_familiy_index that handles graphics and can draw on the surface created by winit
                    q.queue_flags.contains(QueueFlags::GRAPHICS) && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| {
            // lower score for preferred device types
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        })
        .expect("No suitable device found")
}