use tekutonu::{
    controller::GameInput,
    model::GameModel,
    view::{instance::make_instance, GameView},
};
use winit::event_loop::EventLoop;


fn main() {
    let vk = make_instance();

    let event_loop = EventLoop::new();
    let view = GameView::new(vk, event_loop);

    let game = GameModel::default();
    let input = GameInput::new();

    view.run(game, input);
}
