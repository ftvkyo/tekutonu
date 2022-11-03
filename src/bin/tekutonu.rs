use tekutonu::{
    controller::GameInput,
    model::GameModel,
    view::{instance::make_instance, GameView},
};
use tracing_subscriber::fmt::format::FmtSpan;
use winit::event_loop::EventLoop;


fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .init();

    let vk = make_instance();

    let event_loop = EventLoop::new();
    let view = GameView::new(vk, event_loop);

    let game = GameModel::default();
    let input = GameInput::new();

    view.run(game, input);
}
