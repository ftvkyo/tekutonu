use std::time::Instant;

use renderer::Renderer;
use winit::{
    event::{DeviceEvent, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use self::texture::TextureLoader;
use crate::{controller::GameInput, model::GameModel};

pub mod renderer;
pub mod texture;

pub struct GameView {
    renderer: Renderer,
    loader_tex: TextureLoader,

    event_loop: EventLoop<()>,
}

impl GameView {
    pub fn new(renderer: Renderer, loader_tex: TextureLoader, event_loop: EventLoop<()>) -> Self {
        Self {
            renderer,
            loader_tex,
            event_loop,
        }
    }

    pub fn run(self, mut game: GameModel, mut input: GameInput) {
        let Self {
            mut renderer,
            loader_tex,
            event_loop,
        } = self;

        renderer.set_cursor_hidden(true);
        renderer.set_cursor_locked(true).unwrap();

        let texture = loader_tex.load("tex.png");

        let mut last_tick = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            if last_tick.elapsed().as_millis() > 16 {
                if let Some(effect) = input.tick() {
                    game.apply_effect(effect);
                }
                last_tick = Instant::now();
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(_) => renderer.schedule_recreate_swapchain(),
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    } => {
                        if let Some(effect) = input.keyboard(key, state, control_flow) {
                            game.apply_effect(effect);
                        }
                    },
                    _ => (),
                },
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta, .. },
                    ..
                } => {
                    let effect = input.mouse_movement(delta);
                    game.apply_effect(effect);
                },
                Event::RedrawEventsCleared => {
                    let data = renderer.make_draw_data(&game);
                    renderer.draw(&data, &texture);
                },
                _ => (),
            }
        });
    }
}
