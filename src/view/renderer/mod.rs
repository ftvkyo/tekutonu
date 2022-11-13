use std::{f32::consts::FRAC_PI_2, sync::Arc};

use cgmath::{Matrix4, One, Rad, Vector3};
use tracing::instrument;
use vulkano::{
    buffer::{
        cpu_pool::CpuBufferPoolSubbuffer,
        CpuAccessibleBuffer,
        CpuBufferPool,
        TypedBufferAccess, BufferUsage,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        PrimaryAutoCommandBuffer,
        RenderPassBeginInfo,
        SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        PersistentDescriptorSet,
        WriteDescriptorSet,
    },
    device::{Device, Queue},
    instance::Instance,
    memory::allocator::{StandardMemoryAllocator, MemoryUsage},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    swapchain::{
        acquire_next_image,
        AcquireError,
        Swapchain,
        SwapchainCreateInfo,
        SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
};
use vulkano_win::create_surface_from_winit;
use winit::{
    dpi::{PhysicalSize, Size},
    error::ExternalError,
    event_loop::{EventLoop},
    window::{CursorGrabMode, Window},
};

use data::Vertex;
use crate::{
    model::{GameModel},
};

pub mod instance;

mod data;
mod device;
mod framebuffer;
mod pipeline;
mod render_pass;
mod shaders;
mod swapchain;


pub struct Renderer {
    device: Arc<Device>,
    queues: Vec<Arc<Queue>>,
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,

    alloc_memory: Arc<StandardMemoryAllocator>,
    alloc_ds: Arc<StandardDescriptorSetAllocator>,
    alloc_command: Arc<StandardCommandBufferAllocator>,

    pool_uniform: CpuBufferPool<shaders::vs::ty::Data>,

    should_recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}


impl Renderer {
    #[instrument(skip_all)]
    pub fn new(vk: Arc<Instance>, event_loop: &EventLoop<()>) -> Self {
        let window_builder = winit::window::WindowBuilder::new()
            .with_resizable(false)
            .with_inner_size(Size::Physical(PhysicalSize {
                width: 1920,
                height: 1080,
            }))
            // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .with_title("tekutonu");

        let window = Arc::new(window_builder.build(&event_loop).unwrap());
        let surface = create_surface_from_winit(window.clone(), vk.clone()).unwrap();

        let (device, queues) = device::choose_device_and_queue(vk, surface.clone());

        // Allocating color (image) buffers through creating a swapchain.
        let (swapchain, images) =
            swapchain::make_swapchain_and_images(device.clone(), window.clone(), surface);

        // Describe where the output of the graphics pipeline will go by creating a
        // RenderPass.
        let render_pass = render_pass::make_render_pass(device.clone(), swapchain.clone());

        // Loading shaders
        let vs = shaders::vs::load(device.clone()).unwrap();
        let fs = shaders::fs::load(device.clone()).unwrap();

        // Specify what we want the device to do
        let pipeline = pipeline::make_pipeline(device.clone(), render_pass.clone(), vs, fs);

        // Dynamic viewports allow us to recreate just the viewport when the window is
        // resized.
        // Otherwise we would have to recreate the whole pipeline.
        // However, not using a dynamic viewport could allow the driver to optimize some
        // things at the cost of slower resizes.
        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let alloc_memory = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let alloc_ds =
            Arc::new(StandardDescriptorSetAllocator::new(device.clone()));
        let alloc_command = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let pool_uniform = CpuBufferPool::<shaders::vs::ty::Data>::new(
            alloc_memory.clone(),
            BufferUsage {
                uniform_buffer: true,
                ..BufferUsage::empty()
            },
            MemoryUsage::Upload,
        );

        // render_pass only specifies the layout of framebuffers, we need to actually
        // create them. We should create a separate framebuffer for every image.
        let framebuffers = framebuffer::make_framebuffers(
            &images,
            render_pass.clone(),
            &mut viewport,
            alloc_memory.clone(),
        );

        let should_recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // End of initialization.

        Self {
            device,
            queues,
            window,
            swapchain,
            render_pass,
            pipeline,
            viewport,
            framebuffers,

            alloc_memory,
            alloc_ds,
            alloc_command,

            pool_uniform,

            should_recreate_swapchain,
            previous_frame_end,
        }
    }

    fn build_command_buffer(
        &mut self,
        image_num: usize,
        allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set: Arc<PersistentDescriptorSet>,
        vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
        index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,
    ) -> PrimaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary(
            &allocator,
            // Can only use the command buffer with this queue
            self.queues[0].queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    // One item for each attachment in the render pass that have `LoadOp::Clear`
                    // (otherwise None)
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()), // Color
                        Some(1f32.into()),                 // Depth
                    ],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_num].clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .bind_vertex_buffers(0, vertex_buffer)
            .bind_index_buffer(index_buffer.clone())
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        builder.build().unwrap()
    }

    #[instrument(skip_all)]
    fn recreate_swapchain(
        &mut self,
        dimensions: PhysicalSize<u32>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<Framebuffer>>), ()> {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions.into(),
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            // Likely user resizing the window, just retry.
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                return Err(());
            },
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };

        // Framebuffers depend on the images
        let new_framebuffers = framebuffer::make_framebuffers(
            &new_images,
            self.render_pass.clone(),
            &mut self.viewport,
            self.alloc_memory.clone(),
        );

        Ok((new_swapchain, new_framebuffers))
    }

    fn make_uniforms(
        &self,
        game: &GameModel,
    ) -> Arc<CpuBufferPoolSubbuffer<shaders::vs::ty::Data>> {
        let camera = &game.camera;

        let position = camera.position.map(|v| v as f32);
        let direction = camera.get_look().map(|v| v as f32);

        let aspect_ratio =
            self.swapchain.image_extent()[0] as f32 / self.swapchain.image_extent()[1] as f32;

        // Perspective projection matrix
        let proj = cgmath::perspective(
            // 90 degrees
            Rad(FRAC_PI_2),
            aspect_ratio,
            0.01,
            100.0,
        );

        let view =
            Matrix4::look_at_rh(position, position + direction, Vector3::new(0.0, -1.0, 0.0));

        let scale = Matrix4::from_scale(0.5);

        let uniform_data = shaders::vs::ty::Data {
            world: Matrix4::one().into(),
            view: (view * scale).into(),
            proj: proj.into(),
        };

        self.pool_uniform.from_data(uniform_data).unwrap()
    }

    fn make_vertices_and_indices(
        &self,
        game: &GameModel
    ) -> (
        Arc<CpuAccessibleBuffer<[Vertex]>>,
        Arc<CpuAccessibleBuffer<[u16]>>,
    ) {
        let (v, i) = game
            .world
            .get_chunk([0, 0, 0])
            .get_render_data([0.0, 0.0, 0.0]);

        let v = CpuAccessibleBuffer::from_iter(
            &self.alloc_memory,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            v.into_iter().map(|v| Vertex { position: v }),
        )
        .unwrap();

        let i = CpuAccessibleBuffer::from_iter(
            &self.alloc_memory,
            BufferUsage {
                index_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            i.into_iter().map(|i| i as u16),
        )
        .unwrap();

        (v, i)
    }

    #[instrument(skip_all)]
    pub fn set_cursor_locked(&self, locked: bool) -> Result<(), ExternalError> {
        let grab = if locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        let res = self.window.set_cursor_grab(grab);

        if res.is_err() && locked {
            self.window.set_cursor_grab(CursorGrabMode::Confined)
        } else {
            res
        }
    }

    #[instrument(skip_all)]
    pub fn set_cursor_hidden(&self, hidden: bool) {
        self.window.set_cursor_visible(!hidden)
    }

    pub fn schedule_recreate_swapchain(&mut self) {
        self.should_recreate_swapchain = true;
    }

    pub fn make_draw_data(&self, game: &GameModel) -> DrawData {
        let (vertices, indices) = self.make_vertices_and_indices(game);
        let uniforms = self.make_uniforms(game);
        DrawData {
            vertices,
            indices,
            uniforms,
        }
    }
}

pub struct DrawData {
    vertices: Arc<CpuAccessibleBuffer<[Vertex]>>,
    indices: Arc<CpuAccessibleBuffer<[u16]>>,
    uniforms: Arc<CpuBufferPoolSubbuffer<shaders::vs::ty::Data>>,
}

impl Renderer {
    pub fn draw(&mut self, data: &DrawData) {
        // Do not draw frame when screen dimensions are zero.
        // On Windows, this can occur from minimizing the application.
        let dimensions = self.window.inner_size();
        if dimensions.width == 0 || dimensions.height == 0 {
            return;
        }

        // Periodic garbage collection.
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Whenever the window resizes we need to recreate everything dependent on the
        // window size.
        if self.should_recreate_swapchain {
            match self.recreate_swapchain(dimensions) {
                Ok((swapchain, framebuffers)) => {
                    self.swapchain = swapchain;
                    self.framebuffers = framebuffers;
                },
                _ => {
                    return;
                },
            }
            self.should_recreate_swapchain = false;
        }

        let pipeline_layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let descriptor_set = PersistentDescriptorSet::new(
            &self.alloc_ds,
            pipeline_layout.clone(),
            [WriteDescriptorSet::buffer(0, data.uniforms.clone())],
        )
        .unwrap();

        // Acquire image from the swapchain for drawing. Wait if no image is yet
        // available.
        let (image_num, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.should_recreate_swapchain = true;
                    return;
                },
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        // May happen when window is being resized.
        // We can still render, but we should recreate it when we have a chance.
        if suboptimal {
            self.should_recreate_swapchain = true;
        }

        let command_buffer = self.build_command_buffer(
            image_num as usize,
            self.alloc_command.clone(),
            descriptor_set,
            data.vertices.clone(),
            data.indices.clone(),
        );

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queues[0].clone(), command_buffer)
            .unwrap()
            // Submit a present command at the end of the queue.
            .then_swapchain_present(
                self.queues[0].clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    image_num,
                ),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            },
            Err(FlushError::OutOfDate) => {
                self.should_recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            },
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            },
        }
    }
}
