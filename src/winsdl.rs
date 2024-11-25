use sdl2::{
    video::{gl_attr::GLAttr, GLContext, SwapInterval, Window},
    EventPump, Sdl,
};

pub struct WinSdl {
    pub _sdl_context: Sdl,
    pub window: Window,
    pub _gl_context: GLContext,
    pub _gl: (),
    pub event_pump: EventPump,
}

impl WinSdl {
    pub fn new(width: usize, height: usize) -> Result<Self, &'static str> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let gl_attr: GLAttr<'_> = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 6);
        gl_attr.set_double_buffer(true);
        gl_attr.set_framebuffer_srgb_compatible(true);

        let window = video_subsystem
            .window("AMTF Engine", width as u32, height as u32)
            .opengl()
            .resizable()
            .position_centered()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        let gl = gl::load_with(|s| {
            video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
        });

        // Enable vsync
        if let Err(error) = window.subsystem().gl_set_swap_interval(SwapInterval::VSync) {
            println!(
                "Failed to gl_set_swap_interval(SwapInterval::VSync): {}",
                error
            );
        };

        let event_pump = sdl_context.event_pump().unwrap();

        return Ok(WinSdl {
            _sdl_context: sdl_context,
            window,
            _gl_context: gl_context,
            _gl: gl,
            event_pump,
        });
    }
}
