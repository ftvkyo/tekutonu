use tekutonu::{
    controller::GameInput,
    model::GameModel,
    view::{
        renderer::{instance::make_instance, Renderer},
        GameView, texture::TextureLoader,
    },
};
use tracing_subscriber::fmt::format::FmtSpan;
use winit::event_loop::EventLoop;


/// Erase everything in the terminal ;)
fn terminal_clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn main() {
    terminal_clear();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();

    let loader_tex = TextureLoader::new("res/".into());

    let vk = make_instance();
    let event_loop = EventLoop::new();

    let renderer = Renderer::new(vk, &event_loop);
    let view = GameView::new(renderer, loader_tex, event_loop);

    let game = GameModel::default();
    let input = GameInput::new();

    view.run(game, input);
}
