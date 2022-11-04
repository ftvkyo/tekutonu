use std::sync::Arc;

use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline,
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
    GraphicsPipeline::start()
        // Which subpass of which render pass this pipeline is going to be used in.
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        // How the vertices are laid out.
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .build(device)
        .unwrap()
}
