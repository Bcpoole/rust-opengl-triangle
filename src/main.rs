#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;

pub mod render_gl;
pub mod resources;

use render_gl::{buffer, data};
use resources::Resources;
use std::path::Path;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C)]
struct Vertex {
    #[location = 0]
    pos: data::f32x3,
    #[location = 1]
    clr: data::f32x3,
}

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("A fantastic window!");

    let windowed_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|ptr| windowed_context.get_proc_address(ptr) as *const _);

    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();
    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/triangle").unwrap();

    // set up vertex buffer object
    let vertices: Vec<Vertex> = vec![
        Vertex {
            pos: (0.5, -0.5, 0.0).into(),
            clr: (1.0, 0.0, 0.0).into(),
        }, // bottom right
        Vertex {
            pos: (-0.5, -0.5, 0.0).into(),
            clr: (0.0, 1.0, 0.0).into(),
        }, // bottom left
        Vertex {
            pos: (0.0, 0.5, 0.0).into(),
            clr: (0.0, 0.0, 1.0).into(),
        }, // top
    ];

    let vbo = buffer::ArrayBuffer::new(&gl);
    vbo.bind();
    vbo.static_draw_data(&vertices);
    vbo.unbind();

    // set up vertex array object

    let vao = buffer::VertexArray::new(&gl);
    vao.bind();
    vbo.bind();

    Vertex::vertex_attrib_pointers(&gl);

    vbo.unbind();
    vao.unbind();

    // set up shared state for window

    unsafe {
        gl.Viewport(0, 0, 900, 700);
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl.Clear(gl::COLOR_BUFFER_BIT);
                }
                shader_program.set_used();
                vao.bind();
                unsafe {
                    gl.DrawArrays(
                        gl::TRIANGLES, // mode
                        0,             // starting index in the enabled arrays
                        3,             // number of indices to be rendered
                    );
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });

    // if let Err(e) = run() {
    //     println!("{}", failure_to_string(e));
    // }
}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e
        .iter_chain()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
    {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}
