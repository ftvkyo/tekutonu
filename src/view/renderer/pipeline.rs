use std::sync::Arc;

use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::BuffersDefinition,
            viewport::ViewportState, color_blend::ColorBlendState,
        },
        GraphicsPipeline,
        StateMode,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderModule,
};

use super::data::Vertex;

pub fn make_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
) -> Arc<GraphicsPipeline> {
    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::start()
        // How the vertices are laid out.
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .rasterization_state(RasterizationState {
            // polygon_mode: todo!(),
            cull_mode: StateMode::Fixed(CullMode::Back),
            ..Default::default()
        })
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
        // Which subpass of which render pass this pipeline is going to be used in.
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}
