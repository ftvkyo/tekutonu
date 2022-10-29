use tekutonu::rendering::{instance::make_instance, renderer::GameRenderer};


fn main() {
    let instance = make_instance();

    let renderer = GameRenderer::new(instance);

    renderer.render();
}
