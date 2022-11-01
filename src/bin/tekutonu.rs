use tekutonu::{
    controller::InputProcessor,
    model::Game,
    view::{instance::make_instance, GameView},
};
use winit::event_loop::EventLoop;


fn main() {
    let vk = make_instance();

    let event_loop = EventLoop::new();
    let view = GameView::new(vk, event_loop);

    let game = Game::default();
    let input = InputProcessor::new();

    view.run(game, input);
}
