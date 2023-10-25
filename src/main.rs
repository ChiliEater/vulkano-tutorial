use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, RenderPassBeginInfo, SubpassContents};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::single_pass_renderpass;
use vulkano::swapchain::{self, AcquireError, SwapchainPresentInfo};
use vulkano::sync::{self, FlushError, GpuFuture};

use vulkano_tutorial::mesh::vertex::Vertex3d;
use vulkano_tutorial::shaders::simple;
use vulkano_tutorial::vulkano_objects;
use vulkano_tutorial::vulkano_objects::buffers::window_size_dependent_setup;
use vulkano_tutorial::vulkano_objects::pipeline::create_pipeline;
use vulkano_tutorial::vulkano_objects::swapchain::{create_swapchain, swapchain_from};
use vulkano_win::VkSurfaceBuild;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let instance = vulkano_objects::instance::new();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    let (physical_device, queue_family_index) =
        vulkano_objects::device::query_device(instance.clone(), surface.clone());

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

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = create_swapchain(device.clone(), surface.clone());

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    // TODO: Clean this up later
    let vertices = [
        Vertex3d {
            position: [-0.5, 0.5, 0.0],
        },
        Vertex3d {
            position: [0.5, 0.5, 0.0],
        },
        Vertex3d {
            position: [0.0, -0.5, 0.0],
        },
    ];

    let vertices_count = vertices.len();

    let vertex_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: vulkano::memory::allocator::MemoryUsage::Upload,
            ..Default::default()
        },
        vertices,
    )
    .unwrap();

    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    // shaders around here
    let vs_temp = simple::vs::load(device.clone()).unwrap();
    let fs_temp = simple::fs::load(device.clone()).unwrap();

    let render_pass = single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = create_pipeline(
        device.clone(),
        vs_temp,
        fs_temp,
        render_pass.clone(),
        viewport.clone(),
    );

    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                previous_frame_end
                    .as_mut()
                    .take()
                    .unwrap()
                    .cleanup_finished();

                // Handle resize
                if recreate_swapchain {
                    let (new_swapchain, new_images) =
                        swapchain_from(surface.clone(), swapchain.clone());
                    swapchain = new_swapchain;
                    if new_images.is_empty() {
                        return;
                    }
                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                    );
                    recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let clear_values = vec![Some([0.0, 0.68, 1.0, 1.0].into())];

                // Create command buffer stuff
                let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                cmd_buffer_builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values,
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    .set_viewport(0, [viewport.clone()])
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .draw(vertices_count as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                let command_buffer = cmd_buffer_builder.build().unwrap();

                // Execute render command
                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                // Check if GPU future is ok
                match future {
                    Ok(future) => {
                        previous_frame_end = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                }
            }
            _ => {}
        }
    });
}
