use glow::{self, RenderLoop, RenderingContext};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn wasm_main() {
    main();
}

fn main() {
    unsafe {
        // Create a context from a WebGL2 context on wasm32 targets
        #[cfg(target_arch = "wasm32")]
        let (_window, context, _events_loop, render_loop) = {
            use wasm_bindgen::JsCast;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            let webgl2_context = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::WebGl2RenderingContext>()
                .unwrap();
            (
                (),
                glow::WebRenderingContext::from_webgl2_context(webgl2_context),
                (),
                glow::WebRenderLoop::from_request_animation_frame(),
            )
        };

        // Create a context from a glutin window on non-wasm32 targets
        #[cfg(not(target_arch = "wasm32"))]
        let (window, context, mut events_loop, render_loop) = {
            use glutin::GlContext;
            let events_loop = glutin::EventsLoop::new();
            let window_builder = glutin::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0));
            let context_builder = glutin::ContextBuilder::new().with_vsync(true);
            let window =
                glutin::GlWindow::new(window_builder, context_builder, &events_loop).unwrap();
            let context = glow::NativeRenderingContext::from_glutin_window(&window);
            window.make_current().unwrap();
            let window_ref = std::sync::Arc::new(window);
            let render_loop = glow::NativeRenderLoop::from_glutin_window(window_ref.clone());
            (window_ref, context, events_loop, render_loop)
        };

        let vertex_array = context
            .create_vertex_array()
            .expect("Cannot create vertex array");
        context.bind_vertex_array(Some(vertex_array));

        let program = context.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::ShaderType::Vertex, vertex_shader_source),
            (glow::ShaderType::Fragment, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = context
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            // Provide different shader versions for wasm32 and non-wasm32 targets
            #[cfg(target_arch = "wasm32")]
            let version = "#version 300 es";
            #[cfg(not(target_arch = "wasm32"))]
            let version = "#version 410";
            context.shader_source(shader, &format!("{}\n{}", version, shader_source));
            context.compile_shader(shader);
            if !context.get_shader_compile_status(shader) {
                panic!(context.get_shader_info_log(shader));
            }
            context.attach_shader(program, shader);
            shaders.push(shader);
        }

        context.link_program(program);
        if !context.get_program_link_status(program) {
            panic!(context.get_program_info_log(program));
        }

        for shader in shaders {
            context.detach_shader(program, shader);
            context.delete_shader(shader);
        }

        context.use_program(Some(program));

        render_loop.run(move |running: &mut bool| {
            // Handle events differently between targets
            #[cfg(not(target_arch = "wasm32"))]
            events_loop.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => *running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = window.get_hidpi_factor();
                        window.resize(logical_size.to_physical(dpi_factor));
                    }
                    _ => (),
                },
                _ => (),
            });

            context.draw_arrays(glow::PrimitiveMode::Triangles, 0, 3);

            if !*running {
                context.delete_program(program);
                context.delete_vertex_array(vertex_array);
            }
        });
    }
}
