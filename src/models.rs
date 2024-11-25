use core::f32;
use std::{ffi::c_void, path::Path};

use cgmath::{
    num_traits::zero, perspective, Euler, InnerSpace, Matrix4, PerspectiveFov, Point3, Quaternion,
    Rad, Rotation3, Vector2, Vector3, Zero,
};
use egui_sdl2_gl::egui::{self, DragValue, Widget};

use crate::{
    buffers::{FrameBuffer, IndexBuffer, ModelTexture, Texture, VertexArrayBuffer, VertexBuffer}, create_framebuffer_depthbuffer,
    draw_scene_custom_shader_program,
    globals::Globals,
    material_structs::{DirectionalLightInfo, MaterialInfo, PointLightInfo, SpotLightInfo},
    shaders::{create_program, Program},
};

pub struct Model {
    pub vbo: VertexBuffer,
    pub vao: VertexArrayBuffer,
    pub ibo: IndexBuffer,

    pub shader_program: Program,
    pub diffuse_texture: Texture,

    pub vertices: Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)>,
    pub indices: Vec<u32>,

    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,

    pub euler_angles: Euler<Rad<f32>>,
    pub using_euler_angles: bool,

    pub material_info: MaterialInfo,
    pub name: String,
    pub render_shadows: bool,
}
impl Model {
    pub fn new(
        vertices: &Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)>,
        indices: &Vec<u32>,
        vert_shader_path: &str,
        frag_shader_path: &str,
        name: String,
        
    ) -> Self {
        let vbo = VertexBuffer::new();
        let vao = VertexArrayBuffer::new();
        let ibo = IndexBuffer::new();

        return Self {
            vbo: vbo,
            vao: vao,
            ibo: ibo,
            shader_program: create_program(vert_shader_path, frag_shader_path).unwrap(),
            diffuse_texture: Texture::new(),
            vertices: vertices.clone(),
            indices: indices.clone(),
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1., 1., 1.),
            euler_angles: Euler {
                x: Rad(0.),
                y: Rad(0.),
                z: Rad(0.),
            },
            using_euler_angles: false,
            material_info: MaterialInfo {
                ambient: Vector3::new(1.0, 1.0, 1.0),
                diffuse: Vector3::new(1.0, 1.0, 1.0),
                specular: 0.5,
                shininess: 32.0,
            },
            name,
            render_shadows: true,
        };
    }
    pub fn start(&self) {
        self.vbo.set(&self.vertices);
        self.vao.set();
        self.ibo.set(&self.indices);
    }
    pub fn start_render(&mut self) {
        self.shader_program.set();
        self.vbo.bind();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.diffuse_texture.bind_texture();
        self.vao.bind();
        self.ibo.bind();
        if self.using_euler_angles {
            self.rotation = self.euler_angles.into();
        }
    }
    pub fn start_render_custom_shader_program(&mut self, shader_program: &Program) {
        shader_program.set();
        self.vbo.bind();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.diffuse_texture.bind_texture();
        self.vao.bind();
        self.ibo.bind();
        if self.using_euler_angles {
            self.rotation = self.euler_angles.into();
        }
    }

    pub fn render(&self, screen_size: Vector2<f32>, view_matrix: Matrix4<f32>, projection_matrix: Matrix4<f32>) {
        let model_matrix: Matrix4<f32> =
            transforms_to_matrix(self.position, self.rotation, self.scale);
        self.shader_program.set_matrix4_float("model", model_matrix);
        self.shader_program.set_matrix4_float("view", view_matrix);
        self.shader_program
            .set_matrix4_float("projection", projection_matrix);

        self.shader_program.set_bool("renderFullbright", false);
        self.shader_program.set_int("texture1", 1);
        self.shader_program.set_int("texture2", 2);
        self.shader_program.set_int("texture3", 3);
        self.shader_program.set_float("uWidth", screen_size.x);
        self.shader_program.set_float("uHeight", screen_size.y);
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const c_void,
            );
        }
    }

    pub fn render_fullbright(&self, screen_size: Vector2<f32>, view_matrix: Matrix4<f32>, projection_matrix: Matrix4<f32>) {
        let model_matrix: Matrix4<f32> =
            transforms_to_matrix(self.position, self.rotation, self.scale);
        self.shader_program.set_matrix4_float("model", model_matrix);
        self.shader_program.set_matrix4_float("view", view_matrix);
        self.shader_program
            .set_matrix4_float("projection", projection_matrix);

        self.shader_program.set_bool("renderFullbright", true);

        self.shader_program.set_float("uWidth", screen_size.x);
        self.shader_program.set_float("uHeight", screen_size.y);

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const c_void,
            );
        }
    }

    pub fn render_custom_shader_program(
        &self,
        view_matrix: Matrix4<f32>,
        projection_matrix: Matrix4<f32>,
        shader_program: &Program,
    ) {
        let model_matrix: Matrix4<f32> =
            transforms_to_matrix(self.position, self.rotation, self.scale);
        shader_program.set_matrix4_float("model", model_matrix);
        shader_program.set_matrix4_float("view", view_matrix);
        shader_program.set_matrix4_float("projection", projection_matrix);

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const c_void,
            );
        }
    }

    pub fn set_light_render_info(
        &self,
        light_position: Vector3<f32>,
        cam: &Camera,
        light: PointLightInfo,
    ) {
        self.shader_program
            .set_point_light_info("pointLights[0]", light, light_position);

        self.shader_program.set_vector3("viewPos", cam.position);

        self.shader_program
            .set_material_info("material", self.material_info);
    }

    pub fn from_obj_file(obj_file: String) -> Self {
        let mut load_options = tobj::LoadOptions::default();
        load_options.triangulate = true;
        load_options.single_index = true;
        let (models, materials) =
            tobj::load_obj(&obj_file, &load_options).expect("Failed to OBJ load file");

        let mut vertices: Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)> =
            Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Note: If you don't mind missing the materials, you can generate a default.
        let materials = materials.expect("Failed to load MTL file");

        println!("Number of models          = {}", models.len());
        println!("Number of materials       = {}", materials.len());

        for (i, m) in models.iter().enumerate() {
            let mesh = &m.mesh;
            println!("");
            println!("model[{}].name             = \'{}\'", i, m.name);
            println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

            println!("model[{}].face_count       = {}", i, mesh.indices.len() / 3,);
            assert!(mesh.indices.len() % 3 == 0);

            let mut next_face = 0;
            for _face in 0..mesh.indices.len() / 3 {
                let end = next_face + 3 as usize;

                let face_indices = &mesh.indices[next_face..end];
                /*println!(" face[{}].indices          = {:?}", face, face_indices);*/
                indices.extend_from_slice(face_indices);

                /*if !mesh.texcoord_indices.is_empty() {
                    let texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
                    println!(
                        " face[{}].texcoord_indices = {:?}",
                        face, texcoord_face_indices
                    );
                }
                if !mesh.normal_indices.is_empty() {
                    let normal_face_indices = &mesh.normal_indices[next_face..end];
                    println!(
                        " face[{}].normal_indices   = {:?}",
                        face, normal_face_indices
                    );
                }*/

                next_face = end;
            }

            // Normals and texture coordinates are also loaded, but not printed in
            // this example.
            /*println!(
                "model[{}].positions        = {}",
                i,
                mesh.positions.len() / 3
            );*/
            assert!(mesh.positions.len() % 3 == 0);

            /*println!(
                "model[{}].vertex_color        = {}",
                i,
                mesh.vertex_color.len() / 3
            );*/
            assert!(mesh.vertex_color.len() % 3 == 0);

            /*println!(
                "model[{}].texcoords        = {}",
                i,
                mesh.texcoords.len() / 2
            );*/
            assert!(mesh.texcoords.len() % 2 == 0);
            if mesh.vertex_color.len() != 0 {
                for vtx in 0..mesh.positions.len() / 3 {
                    vertices.push((
                        Vector3::new(
                            mesh.positions[3 * vtx],
                            mesh.positions[3 * vtx + 1],
                            mesh.positions[3 * vtx + 2],
                        ),
                        Vector3::new(
                            mesh.vertex_color[3 * vtx],
                            mesh.vertex_color[3 * vtx + 1],
                            mesh.vertex_color[3 * vtx + 2],
                        ),
                        Vector3::new(
                            mesh.normals[3 * vtx],
                            mesh.normals[3 * vtx + 1],
                            mesh.normals[3 * vtx + 2],
                        ),
                        Vector2::new(mesh.texcoords[2 * vtx], mesh.texcoords[2 * vtx + 1]),
                    ));
                    println!(
                        "              position[{}] = ({}, {}, {})",
                        vtx,
                        mesh.positions[3 * vtx],
                        mesh.positions[3 * vtx + 1],
                        mesh.positions[3 * vtx + 2]
                    );
                    println!(
                        "              vertex_color[{}] = ({}, {}, {})",
                        vtx,
                        mesh.vertex_color[3 * vtx],
                        mesh.vertex_color[3 * vtx + 1],
                        mesh.vertex_color[3 * vtx + 2]
                    );
                    println!(
                        "              texcoords[{}] = ({}, {})",
                        vtx,
                        mesh.texcoords[2 * vtx],
                        mesh.texcoords[2 * vtx + 1],
                    );
                    println!(
                        "              normals[{}] = ({}, {}, {})",
                        vtx,
                        mesh.normals[3 * vtx],
                        mesh.normals[3 * vtx + 1],
                        mesh.normals[3 * vtx + 2]
                    );
                }
            } else {
                for vtx in 0..mesh.positions.len() / 3 {
                    vertices.push((
                        Vector3::new(
                            mesh.positions[3 * vtx],
                            mesh.positions[3 * vtx + 1],
                            mesh.positions[3 * vtx + 2],
                        ),
                        Vector3::new(1., 1., 1.),
                        Vector3::new(
                            mesh.normals[3 * vtx],
                            mesh.normals[3 * vtx + 1],
                            mesh.normals[3 * vtx + 2],
                        ),
                        Vector2::new(mesh.texcoords[2 * vtx], mesh.texcoords[2 * vtx + 1]),
                    ));
                    println!(
                        "              position[{}] = ({}, {}, {})",
                        vtx,
                        mesh.positions[3 * vtx],
                        mesh.positions[3 * vtx + 1],
                        mesh.positions[3 * vtx + 2]
                    );
                    println!(
                        "              vertex_color[{}] = ({}, {}, {})",
                        vtx, 1., 1., 1.
                    );
                    println!(
                        "              texcoords[{}] = ({}, {})",
                        vtx,
                        mesh.texcoords[2 * vtx],
                        mesh.texcoords[2 * vtx + 1],
                    );
                    println!(
                        "              normals[{}] = ({}, {}, {})",
                        vtx,
                        mesh.normals[3 * vtx],
                        mesh.normals[3 * vtx + 1],
                        mesh.normals[3 * vtx + 2]
                    );
                }
            }
        }

        let mut diffuse_texture_path: String = String::new();

        for (i, m) in materials.iter().enumerate() {
            println!("material[{}].name = \'{}\'", i, m.name);
            println!(
                "    material.Ka = ({}, {}, {})",
                m.ambient.unwrap()[0],
                m.ambient.unwrap()[1],
                m.ambient.unwrap()[2]
            );
            if m.diffuse.is_some() {
                println!(
                    "    material.Kd = ({}, {}, {})",
                    m.diffuse.unwrap()[0],
                    m.diffuse.unwrap()[1],
                    m.diffuse.unwrap()[2]
                );
            }
            println!(
                "    material.Ks = ({}, {}, {})",
                m.specular.unwrap()[0],
                m.specular.unwrap()[1],
                m.specular.unwrap()[2]
            );
            println!("    material.Ns = {}", m.shininess.unwrap());
            println!("    material.d = {}", m.dissolve.unwrap());
            if m.ambient_texture.is_some() {
                println!(
                    "    material.map_Ka = {}",
                    m.ambient_texture.clone().unwrap()
                );
            }
            if m.diffuse_texture.is_some() {
                println!(
                    "    material.map_Kd = {}",
                    m.diffuse_texture.clone().unwrap()
                );
                diffuse_texture_path = m.diffuse_texture.clone().unwrap();
                diffuse_texture_path = diffuse_texture_path.replacen("..", ".", 1);
            }
            if m.specular_texture.is_some() {
                println!(
                    "    material.map_Ks = {}",
                    m.specular_texture.clone().unwrap()
                );
            }
            if m.shininess_texture.is_some() {
                println!(
                    "    material.map_Ns = {}",
                    m.shininess_texture.clone().unwrap()
                );
            }
            if m.normal_texture.is_some() {
                println!(
                    "    material.map_Bump = {}",
                    m.normal_texture.clone().unwrap()
                );
            }
            if m.dissolve_texture.is_some() {
                println!(
                    "    material.map_d = {}",
                    m.dissolve_texture.clone().unwrap()
                );
            }

            for (k, v) in &m.unknown_param {
                println!("    material.{} = {}", k, v);
            }
        }

        println!("{:#?}", vertices);
        println!("{:#?}", indices);

        let name = Path::new(&obj_file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".obj", "");
        let texture = Texture::new();
        texture.load(&Path::new(&diffuse_texture_path)).unwrap();
        let mut model = Model::new(
            &vertices,
            &indices,
            "./shaders/BasicModel/shader.vert",
            "./shaders/BasicModel/shader.frag",
            name,
        );
        model.diffuse_texture = texture;

        model.start();

        return model;
    }
}

struct EulerWidget<'a> {
    euler: &'a mut Euler<Rad<f32>>,
    using: &'a mut bool,
}

impl<'a> From<(&'a mut Euler<Rad<f32>>, &'a mut bool)> for EulerWidget<'a> {
    fn from(value: (&'a mut Euler<Rad<f32>>, &'a mut bool)) -> Self {
        return Self {
            euler: value.0,
            using: value.1,
        };
    }
}

impl<'a> Widget for &mut EulerWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .indent("", |ui| {
                ui.add(
                    egui_sdl2_gl::egui::DragValue::new(&mut self.euler.x.0)
                        .prefix("X ")
                        .suffix(" Rad")
                        .speed(if *self.using { 0.01 } else { 0. })
                        .fixed_decimals(3),
                );
                ui.add(
                    egui::DragValue::new(&mut self.euler.y.0)
                        .prefix("Y ")
                        .suffix(" Rad")
                        .speed(if *self.using { 0.01 } else { 0. })
                        .fixed_decimals(3),
                );
                ui.add(
                    egui::DragValue::new(&mut self.euler.z.0)
                        .prefix("Z ")
                        .suffix(" Rad")
                        .speed(if *self.using { 0.01 } else { 0. })
                        .fixed_decimals(3),
                );
                ui.checkbox(&mut self.using, "Using Euler Angles?");
            })
            .response;
        return response;
    }
}

struct Vector3Widget<'a> {
    vector3: &'a mut Vector3<f32>,
    scale: bool,
}

impl<'a> From<&'a mut Vector3<f32>> for Vector3Widget<'a> {
    fn from(value: &'a mut Vector3<f32>) -> Self {
        return Self {
            vector3: value,
            scale: false,
        };
    }
}

impl<'a> Widget for &mut Vector3Widget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .indent("", |ui| {
                let mut x_widget = egui_sdl2_gl::egui::DragValue::new(&mut self.vector3.x)
                    .prefix("X ")
                    .speed(0.01);
                let mut y_widget = egui_sdl2_gl::egui::DragValue::new(&mut self.vector3.y)
                    .prefix("Y ")
                    .speed(0.01);
                let mut z_widget = egui_sdl2_gl::egui::DragValue::new(&mut self.vector3.z)
                    .prefix("Z ")
                    .speed(0.01);
                if !self.scale {
                    x_widget = x_widget.suffix(" m");
                    y_widget = y_widget.suffix(" m");
                    z_widget = z_widget.suffix(" m");
                }
                ui.add(x_widget);
                ui.add(y_widget);
                ui.add(z_widget);
            })
            .response;
        return response;
    }
}

struct Color3Widget<'a> {
    vector3: &'a mut Vector3<f32>,
}

impl<'a> From<&'a mut Vector3<f32>> for Color3Widget<'a> {
    fn from(value: &'a mut Vector3<f32>) -> Self {
        return Self { vector3: value };
    }
}

impl<'a> Widget for &mut Color3Widget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .indent("", |ui| {
                let x_widget = egui_sdl2_gl::egui::DragValue::new(&mut self.vector3.x)
                    .prefix("R ")
                    .speed(0.01);
                let y_widget = egui_sdl2_gl::egui::DragValue::new(&mut self.vector3.y)
                    .prefix("G ")
                    .speed(0.01);
                let z_widget = egui_sdl2_gl::egui::DragValue::new(&mut self.vector3.z)
                    .prefix("B ")
                    .speed(0.01);
                ui.add(x_widget);
                ui.add(y_widget);
                ui.add(z_widget);
            })
            .response;
        return response;
    }
}

impl Widget for &mut Model {
    fn ui(self, ui: &mut egui_sdl2_gl::egui::Ui) -> egui_sdl2_gl::egui::Response {
        let mut euler_angles: Euler<Rad<f32>>;
        if !self.using_euler_angles {
            euler_angles = self.rotation.into();
        } else {
            euler_angles = self.euler_angles;
        }
        let mut position_widget: Vector3Widget = Vector3Widget::from(&mut self.position);
        let mut rotation_widget: EulerWidget =
            EulerWidget::from((&mut euler_angles, &mut self.using_euler_angles));
        let mut scale_widget: Vector3Widget = Vector3Widget::from(&mut self.scale);
        scale_widget.scale = true;
        let response = ui
            .group(|ui| {
                ui.label(self.name.clone());
                ui.indent("", |ui| {
                    ui.label("Position");
                    ui.add(&mut position_widget);
                    ui.label("Rotaton");
                    ui.add(&mut rotation_widget);
                    ui.label("Scale");
                    ui.add(&mut scale_widget);
                });
            })
            .response;
        self.euler_angles = euler_angles;
        return response;
    }
}

pub struct Camera {
    pub projection_matrix: PerspectiveFov<f32>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub camera_rotation: Vector2<f32>,
}

impl Camera {
    pub fn new(aspect_ratio: f32, fovy: f32, near: f32, far: f32) -> Self {
        Self {
            projection_matrix: PerspectiveFov {
                fovy: cgmath::Rad(fovy),
                aspect: aspect_ratio,
                near,
                far,
            },
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3 {
                x: 1.,
                y: 1.,
                z: 1.,
            },
            camera_rotation: Vector2::zero(),
        }
    }

    pub fn view_transform(&mut self) -> Matrix4<f32> {
        self.camera_rotation.y = self
            .camera_rotation
            .y
            .clamp(-90.0f32.to_radians(), 90.0f32.to_radians());
        self.rotation = Quaternion::from_angle_y(Rad(self.camera_rotation.x))
            * Quaternion::from_angle_x(Rad(self.camera_rotation.y));
        transforms_to_matrix(self.position, self.rotation, self.scale)
    }
}

fn transforms_to_matrix(
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
) -> Matrix4<f32> {
    let position_matrix: Matrix4<f32> = Matrix4::from_translation(position);
    let rotation_matrix: Matrix4<f32> = rotation.into();
    let scale_matrix: Matrix4<f32> = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
    position_matrix * scale_matrix * rotation_matrix
}

pub struct DirectionalLight {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,

    pub info: DirectionalLightInfo,
    pub shadow_texture: Texture,
    pub shadow_framebuffer: FrameBuffer,
    pub resolution: (u32, u32),
    pub light_projection: Matrix4<f32>,
    pub light_view: Matrix4<f32>,
}

impl DirectionalLight {
    pub fn new(shadow_resolution: (u32, u32)) -> Self {
        let direction: Vector3<f32> = Vector3::new(0., -1., 0.);

        let info = DirectionalLightInfo {
            color: Vector3::new(1.0, 1.0, 1.0),
        };

        let (shadow_texture, shadow_framebuffer) =
            create_framebuffer_depthbuffer(shadow_resolution);
        let (near_plane, far_plane) = (0.1f32, 100.0f32);
        let light_projection =
            cgmath::ortho(-10.0f32, 10.0f32, -10.0f32, 10.0f32, near_plane, far_plane);
        let light_pos = Vector3::zero();
        let light_pos_point = Point3::new(light_pos.x, light_pos.y, light_pos.z);
        let light_view: Matrix4<f32> = cgmath::Matrix4::look_at_rh(
            light_pos_point,
            light_pos_point + direction,
            Vector3::unit_y(),
        );

        Self {
            position: Vector3::zero(),
            direction,
            info,
            shadow_texture,
            shadow_framebuffer,
            resolution: shadow_resolution,
            light_projection,
            light_view,
        }
    }
    pub fn render(&mut self, globals: &mut Globals, depth_only_shader: &Program) {
        let (near_plane, far_plane) = (0.01f32, 200.0f32);
        let light_projection = cgmath::ortho(
            -far_plane / 2.0,
            far_plane / 2.0,
            -far_plane / 2.0,
            far_plane / 2.0,
            near_plane,
            far_plane,
        );
        let mut light_pos = globals.cam.position - self.direction * far_plane / 2.0;
        light_pos.x = light_pos.x.round();
        light_pos.y = light_pos.y.round();
        light_pos.z = light_pos.z.round();
        let light_pos_point = Point3::new(light_pos.x, light_pos.y, light_pos.z);
        let light_view: Matrix4<f32> = cgmath::Matrix4::look_at_rh(
            light_pos_point,
            light_pos_point + self.direction,
            Vector3::unit_y(),
        );
        self.light_projection = light_projection;
        self.light_view = light_view;
        unsafe {
            gl::Viewport(0, 0, self.resolution.0 as i32, self.resolution.1 as i32);
        }
        self.shadow_framebuffer.bind();
        draw_scene_custom_shader_program(globals, light_projection, light_view, depth_only_shader, true);
        self.shadow_framebuffer.unbind();
    }
}

impl Widget for &mut DirectionalLight {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut direction_thing: Vector3<f32> = self.direction;
        let mut position_widget: Vector3Widget = Vector3Widget::from(&mut self.position);
        let mut direction_widget: Vector3Widget = Vector3Widget::from(&mut direction_thing);
        let mut color_widget: Color3Widget = Color3Widget::from(&mut self.info.color);
        let response = ui
            .group(|ui| {
                ui.label("Position");
                ui.add(&mut position_widget);
                ui.label("Direction");
                ui.add(&mut direction_widget);
                ui.label("Color");
                ui.add(&mut color_widget);
            })
            .response;
        self.direction = direction_thing.normalize();
        return response;
    }
}

pub struct SpotLight {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub horizontal_fov: f32,
    pub info: SpotLightInfo,

    pub shadow_texture: Texture,
    pub shadow_framebuffer: FrameBuffer,
    pub resolution: (u32, u32),
    pub light_projection: Matrix4<f32>,
    pub light_view: Matrix4<f32>,
}

impl SpotLight {
    pub fn new(shadow_resolution: (u32, u32), horizontal_fov: f32) -> Self {
        let direction: Vector3<f32> = Vector3::new(0., -1., 0.);

        let info = SpotLightInfo {
            color: Vector3::new(1.0, 1.0, 1.0),
            radius: 10.0,
        };

        let (shadow_texture, shadow_framebuffer) =
            create_framebuffer_depthbuffer(shadow_resolution);
        let (near_plane, far_plane) = (0.01f32, info.radius);
        let fovy = 0.5 * shadow_resolution.1 as f32
            / (0.5 * shadow_resolution.0 as f32 / (0.5 * horizontal_fov).tan());
        let light_projection = perspective(
            Rad(fovy),
            shadow_resolution.0 as f32 / shadow_resolution.1 as f32,
            near_plane,
            far_plane,
        );
        let light_pos = Vector3::zero();
        let light_pos_point = Point3::new(light_pos.x, light_pos.y, light_pos.z);
        let light_view: Matrix4<f32> = cgmath::Matrix4::look_at_rh(
            light_pos_point,
            light_pos_point + direction,
            Vector3::unit_y(),
        );

        Self {
            position: Vector3::zero(),
            direction,
            horizontal_fov,
            info,
            shadow_texture,
            shadow_framebuffer,
            resolution: shadow_resolution,
            light_projection,
            light_view,
        }
    }
    pub fn render(&mut self, globals: &mut Globals, depth_only_shader: &Program) {
        let (near_plane, far_plane) = (0.01f32, self.info.radius);
        let fovy = self.horizontal_fov * self.resolution.1 as f32 / self.resolution.0 as f32;
        let light_projection = perspective(
            Rad(fovy),
            self.resolution.0 as f32 / self.resolution.1 as f32,
            near_plane,
            far_plane,
        );
        let light_pos = self.position;
        let light_pos_point = Point3::new(light_pos.x, light_pos.y, light_pos.z);
        let light_view: Matrix4<f32> = cgmath::Matrix4::look_at_rh(
            light_pos_point,
            light_pos_point + self.direction,
            Vector3::unit_y(),
        );
        self.light_projection = light_projection;
        self.light_view = light_view;
        unsafe {
            gl::Viewport(0, 0, self.resolution.0 as i32, self.resolution.1 as i32);
        }
        self.shadow_framebuffer.bind();
        draw_scene_custom_shader_program(globals, light_projection, light_view, depth_only_shader, true);
        self.shadow_framebuffer.unbind();
    }
}

impl Widget for &mut SpotLight {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut direction_thing: Vector3<f32> = self.direction;
        let mut position_widget: Vector3Widget = Vector3Widget::from(&mut self.position);
        let mut direction_widget: Vector3Widget = Vector3Widget::from(&mut direction_thing);
        let mut color_widget: Color3Widget = Color3Widget::from(&mut self.info.color);
        let response = ui
            .group(|ui| {
                ui.label("Position");
                ui.add(&mut position_widget);
                ui.label("Direction");
                ui.add(&mut direction_widget);
                ui.label("Color");
                ui.add(&mut color_widget);
                ui.label("Radius");
                ui.add(
                    DragValue::new(&mut self.horizontal_fov)
                        .suffix(" Rad")
                        .speed(0.01)
                        .fixed_decimals(3),
                )
            })
            .response;
        self.direction = direction_thing.normalize();
        return response;
    }
}

pub struct PointLight {
    pub position: Vector3<f32>,
    pub info: PointLightInfo,
}

impl PointLight {
    pub fn new() -> Self {
        let info = PointLightInfo {
            color: Vector3::new(1.0, 1.0, 1.0),
            radius: 10.0,
        };

        Self {
            position: zero(),
            info,
        }
    }
}

impl Widget for &mut PointLight {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut position_widget: Vector3Widget = Vector3Widget::from(&mut self.position);
        let mut color_widget: Color3Widget = Color3Widget::from(&mut self.info.color);
        let response = ui
            .group(|ui| {
                ui.label("Position");
                ui.add(&mut position_widget);
                ui.label("Color");
                ui.add(&mut color_widget);
            })
            .response;
        return response;
    }
}

//// WIP NOT WORKING
/*pub struct Portal {
    pub model_index: usize,
    pub connected_portal_index: usize,

    pub portal_texture: Texture,
    pub portal_framebuffer: FrameBuffer,
    pub resolution: (u32, u32),
    pub portal_projection: Matrix4<f32>,
    pub portal_view: Matrix4<f32>,
}

impl Portal {
    pub fn new(model_index: usize, connected_portal_index: usize, globals: &mut Globals) -> Self {
        let (portal_texture, portal_framebuffer) =
            create_framebuffer(globals.win_sdl.window.size());

        let portal_projection = perspective(
            globals.cam.projection_matrix.fovy,
            globals.cam.projection_matrix.aspect,
            globals.cam.projection_matrix.near,
            globals.cam.projection_matrix.far,
        );
        let portal_pos = globals.models[model_index].position;
        let portal_pos_point = Point3::new(portal_pos.x, portal_pos.y, portal_pos.z);
        let portal_view: Matrix4<f32> = cgmath::Matrix4::look_at_rh(
            portal_pos_point,
            portal_pos_point + globals.cam.position,
            Vector3::unit_y(),
        );
        return Portal {
            model_index,
            connected_portal_index,
            portal_texture,
            portal_framebuffer,
            resolution: globals.win_sdl.window.size(),
            portal_projection,
            portal_view,
        };
    }
    pub fn render(
        &mut self,
        globals: &mut Globals,
        connected_portal: &mut Self,
        depth: usize,
        maxdepth: usize,
        current_index: usize,
    ) {
        if depth >= maxdepth {
            return;
        }
        connected_portal.connected_portal_index = current_index;
        self.resolution = globals.win_sdl.window.size();
        self.portal_projection = perspective(
            globals.cam.projection_matrix.fovy,
            globals.cam.projection_matrix.aspect,
            globals.cam.projection_matrix.near,
            globals.cam.projection_matrix.far,
        );
        let portal_pos = globals.models[self.model_index].position;
        let portal_pos_point = Point3::new(portal_pos.x, portal_pos.y, portal_pos.z);
        let connected_portal_pos =
            globals.models[connected_portal.model_index].position;
        let connected_portal_pos_point = Point3::new(
            connected_portal_pos.x,
            connected_portal_pos.y,
            connected_portal_pos.z,
        );
        self.portal_view = cgmath::Matrix4::look_at_rh(
            portal_pos_point,
            connected_portal_pos_point + (globals.cam.position - portal_pos),
            Vector3::unit_y(),
        );
        unsafe {
            gl::Viewport(0, 0, self.resolution.0 as i32, self.resolution.1 as i32);
        }
        self.portal_framebuffer.bind();
        draw_scene(globals, self.portal_projection, self.portal_view);
        self.portal_framebuffer.unbind();
    }
} */