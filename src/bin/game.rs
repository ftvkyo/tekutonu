use tekutonu::rendering::{instance::make_instance, renderer::GameRenderer};
use winit::event_loop::EventLoop;


fn main() {
    let instance = make_instance();

    let event_loop = EventLoop::new();
    let renderer = GameRenderer::new(instance, &event_loop);

    renderer.render(event_loop);
}
