use cgmath::{Euler, Quaternion, Rad, Vector2, Vector3};
use egui_sdl2_gl::{egui, painter::Painter, DpiScaling, EguiStateHandler, ShaderVersion};

use crate::{
    models::{Camera, Model},
    shaders::create_program,
    winsdl::WinSdl,
};
/// A bunch of variables that most things need to some extent
pub struct Globals {
    /// basic sld2 things
    pub win_sdl: WinSdl,
    /// egui context, gui context
    pub egui_ctx: egui::Context,
    /// egui painter, it basically puts the gui on screen
    pub egui_painter: Painter,
    /// egui state, the state of the gui
    pub egui_state: EguiStateHandler,
    pub movement_speed: f32,
    pub look_sensitivity: f32,
    pub mouse_look_sensitivity: f32,
    pub should_grab_mouse: bool,
    pub cam: Camera,
    pub models: Vec<Model>,
    /// a model that is a quad that will cover the entire screen
    pub screen_model: Model,
    /// a model that will be rendered where lights are
    pub light_model: Model,
}
///Window Width and height, should probably be variables to new, but I'm commenting rn
const WIDTH: usize = 1920;
const HEIGHT: usize = 1052;
impl Globals {
    pub fn new() -> Self {
        let win_sdl: WinSdl = WinSdl::new(WIDTH, HEIGHT).unwrap();

        let (egui_painter, egui_state) = egui_sdl2_gl::with_sdl2(
            &win_sdl.window,
            ShaderVersion::Adaptive,
            DpiScaling::Default,
        );
        let egui_ctx: egui::Context = egui::Context::default();

        let movement_speed: f32 = 7.0;
        let look_sensitivity: f32 = 1.0;
        let mouse_look_sensitivity: f32 = 1.0;

        let horizontal_fov = 90.0f32.to_radians();
        let fovy = 0.5 * HEIGHT as f32 / (0.5 * WIDTH as f32 / (0.5 * horizontal_fov).tan());

        let cam = Camera::new(WIDTH as f32 / HEIGHT as f32, fovy, 0.1, 100.);

        let models: Vec<Model> = Vec::new();

        let mut light_model = Model::from_obj_file("./models/Cube.obj".to_owned());
        light_model.shader_program = create_program(
            "./shaders/LightSource/shader.vert",
            "./shaders/LightSource/shader.frag",
        )
        .unwrap();
        light_model.position = Vector3::new(2.4, 2.0, 4.0);
        light_model.scale = Vector3::new(0.2, 0.2, 0.2);

        let screen_model_vertices: Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)> = vec![
            (
                Vector3::new(-1., -1., -1.),
                Vector3::new(0., 0., 0.),
                Vector3::new(0., 0., 0.),
                Vector2::new(0., 0.),
            ),
            (
                Vector3::new(-1., 1., -1.),
                Vector3::new(0., 0., 0.),
                Vector3::new(0., 0., 0.),
                Vector2::new(0., 1.),
            ),
            (
                Vector3::new(1., -1., -1.),
                Vector3::new(0., 0., 0.),
                Vector3::new(0., 0., 0.),
                Vector2::new(1., 0.),
            ),
            (
                Vector3::new(1., 1., -1.),
                Vector3::new(0., 0., 0.),
                Vector3::new(0., 0., 0.),
                Vector2::new(1., 1.),
            ),
        ];
        let screen_model_indices: Vec<u32> = vec![0, 1, 2, 1, 2, 3];
        let screen_model = Model::new(
            &screen_model_vertices,
            &screen_model_indices,
            "./shaders/FinalPass/shader.vert",
            "./shaders/FinalPass/shader.frag",
            "Screen Model".to_owned(),
        );
        screen_model.start();

        return Self {
            win_sdl,
            egui_ctx,
            egui_painter,
            egui_state,
            movement_speed,
            look_sensitivity,
            mouse_look_sensitivity,
            should_grab_mouse: true,
            cam,
            models,
            screen_model,
            light_model,
        };
    }
}
