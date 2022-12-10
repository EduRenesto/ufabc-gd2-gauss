use glow::HasContext;
use viewer::Viewer;

mod gfx;
mod viewer;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_major_version(4);

    let win = video
        .window("Gauss", 1280, 720)
        .opengl()
        .build()
        .unwrap();

    let _gl_ctx = win.gl_create_context().unwrap();
    let gl = unsafe { glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _) };

    unsafe {
        gl.enable(glow::DEPTH_TEST);
    }

    //let shader = crate::gfx::Shader::new(
    //    &gl,
    //    include_str!("../res/shaders/simple.frag.glsl"),
    //    include_str!("../res/shaders/simple.vert.glsl"),
    //).unwrap();

    let viewer = Viewer::new(&gl);

    let mut evt_loop = sdl.event_pump().unwrap();

    'main: loop {
        for evt in evt_loop.poll_iter() {
            match evt {
                sdl2::event::Event::Quit { .. } => { break 'main; }
                _ => {  }
            }
        }

        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            viewer.render();
        }

        win.gl_swap_window();
    }
}
