#![cfg_attr(not(target_arch = "wasm32"), windows_subsystem = "windows")]

mod state;
mod texture;
mod vertex;

use state::State;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowAttributes},
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

// ---- Window ----

#[cfg(not(target_arch = "wasm32"))]
const INITIAL_WIDTH: u32 = 1024;
#[cfg(not(target_arch = "wasm32"))]
const INITIAL_HEIGHT: u32 = 768;

// ---- Input ----

const MOUSE_SENSITIVITY: f64 = 0.006;
const ZOOM_SPEED: f32 = 0.5;
const ROTATION_THRESHOLD: f64 = 0.0001;

#[cfg(not(target_arch = "wasm32"))]
const PIXEL_DELTA_SCALE: f32 = 0.1;
#[cfg(target_arch = "wasm32")]
const PIXEL_DELTA_SCALE: f32 = 0.02;

// ---- Camera ----

const MIN_DISTANCE: f32 = 1.5;
const MAX_DISTANCE: f32 = 10.0;

// ---- Entry Points ----

#[cfg(target_arch = "wasm32")]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).unwrap();
    wasm_bindgen_futures::spawn_local(run_wasm());
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init();
    run_desktop();
}

// ---- Desktop ----

#[cfg(not(target_arch = "wasm32"))]
fn run_desktop() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        state: None,
    };
    event_loop.run_app(&mut app).unwrap();
}

// ---- WASM ----

#[cfg(target_arch = "wasm32")]
#[allow(deprecated)]
async fn run_wasm() {
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowAttributesExtWebSys;

    let event_loop = EventLoop::new().unwrap();

    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("canvas"))
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("canvas element not found");

    let window_attrs = WindowAttributes::default()
        .with_title("3D Cube - 6 Photos")
        .with_canvas(Some(canvas));
    let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

    let state = State::new(Arc::clone(&window)).await;
    let app = App {
        window: Some(window),
        state: Some(state),
    };
    event_loop.spawn_app(app);
}

// ---- App ----

struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<State<'a>>,
}

impl<'a> ApplicationHandler for App<'a> {
    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("3D Cube - 6 Photos")
            .with_inner_size(winit::dpi::LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        let state = pollster::block_on(State::new(Arc::clone(&window)));

        self.window = Some(window);
        self.state = Some(state);
    }

    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        let state = self.state.as_mut().unwrap();

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                if state.mouse_pressed {
                    state.mouse_pressed = false;
                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }
            }

            WindowEvent::Resized(physical_size) => state.resize(physical_size),

            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                if button == MouseButton::Left {
                    let pressed = btn_state == ElementState::Pressed;
                    state.mouse_pressed = pressed;
                    if pressed {
                        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                        window.set_cursor_visible(false);
                    } else {
                        let _ = window.set_cursor_grab(CursorGrabMode::None);
                        window.set_cursor_visible(true);
                    }
                }
            }

            WindowEvent::Focused(false) => {
                if state.mouse_pressed {
                    state.mouse_pressed = false;
                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * PIXEL_DELTA_SCALE,
                };
                state.distance =
                    (state.distance - dy * ZOOM_SPEED).clamp(MIN_DISTANCE, MAX_DISTANCE);
            }

            WindowEvent::RedrawRequested => {
                state.update_uniforms();
                state.render();
                window.set_title(&format!("3D Cube - 6 Photos | FPS: {}", state.fps));
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            if state.mouse_pressed {
                let dx = delta.0 * MOUSE_SENSITIVITY;
                let dy = -delta.1 * MOUSE_SENSITIVITY;
                let angle = (dx * dx + dy * dy).sqrt() as f32;
                if angle > ROTATION_THRESHOLD as f32 {
                    let axis = glam::Vec3::new(-dy as f32, dx as f32, 0.0).normalize();
                    let rot = glam::Quat::from_axis_angle(axis, angle);
                    state.rotation = rot * state.rotation;
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
