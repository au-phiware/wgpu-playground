use crate::gpu::GpuContext;
use crate::renderer::Renderer;
use std::sync::Arc;

use anyhow::Result;
use winit::{
    application::ApplicationHandler, event::*, event_loop::ActiveEventLoop,
    keyboard::Key,
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<AppState>>,
    window: Option<Arc<Window>>,
    state: Option<AppState>,
    keyboard_modifiers: winit::keyboard::ModifiersState,
}

pub struct AppState {
    gpu: GpuContext,
    renderer: Renderer,
}

impl AppState {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let gpu = GpuContext::new(window).await?;

        let renderer = Renderer::new(&"Main", &gpu);

        Ok(Self {
            gpu,
            renderer,
        })
    }
}

impl App {
    pub fn new(
        #[cfg(target_arch = "wasm32")] event_loop: &winit::event_loop::EventLoop<AppState>,
    ) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy: Some(event_loop.create_proxy()),
            window: None,
            state: None,
            keyboard_modifiers: Default::default(),
        }
    }
}

impl ApplicationHandler<AppState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let browser = wgpu::web_sys::window().unwrap_throw();
            let document = browser.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.window = Some(window.clone());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the window
            self.state = Some(pollster::block_on(AppState::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                AppState::new(window)
                                    .await
                                    .expect("Unable to create canvas!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    // This is where proxy.send_event() ends up
    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut state: AppState) {
        state.gpu.resize();
        state.gpu.surface.window.request_redraw();

        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match (&mut self.state, event) {
            (_, WindowEvent::CloseRequested) => event_loop.exit(),
            (Some(state), WindowEvent::Resized(_)) => state.gpu.resize(),
            (Some(state), WindowEvent::RedrawRequested) => {
                match state.renderer.render(&state.gpu.surface.window, &state.gpu) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.gpu.resize();
                    }
                    Err(e) => {
                        log::error!("Unable to render {e}");
                    }
                }
            }
            (_, WindowEvent::ModifiersChanged(modifiers)) => {
                self.keyboard_modifiers = modifiers.state();
            }
            (state, WindowEvent::KeyboardInput { event, .. }) => {
                if self.keyboard_modifiers.control_key() && event.state == winit::event::ElementState::Pressed {
                    match event.logical_key {
                        Key::Character(ref key) if key == &"q" => {
                            #[cfg(target_arch = "wasm32")]
                            if let Some(window) = web_sys::window() {
                                let _ = window.close();
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                event_loop.exit();
                            }
                        }
                        Key::Character(ref key) if key == &"r" => {
                            #[cfg(target_arch = "wasm32")]
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().reload();
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if let (Some(state), Some(window)) = (state, &self.window) {
                                state.renderer = Renderer::new(&"Main", &state.gpu);
                                window.request_redraw();
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
