use tekutonu::{
    controller::GameInput,
    model::GameModel,
    view::{instance::make_instance, GameView},
};
use tracing_subscriber::fmt::format::FmtSpan;
use winit::event_loop::EventLoop;


fn main() {
    // Erase everything in the terminal ;)
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

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
