mod buffers;
mod material_structs;
mod models;
mod shaders;
mod winsdl;
mod globals;

use buffers::{Cubemap, FrameBuffer, ModelTexture, RenderBuffer, Texture};
use cgmath::{
    InnerSpace, Matrix4, Quaternion, Rad, Rotation, Rotation3, SquareMatrix, Vector2, Vector3, Zero,
};
use core::f32;
use egui_sdl2_gl::egui;
use egui_sdl2_gl::egui::FullOutput;
use globals::Globals;
use models::{DirectionalLight, PointLight, SpotLight};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use shaders::{create_program, Program};
use std::ptr::null;
use std::time::Instant;

pub fn main() {
    let mut globals = Globals::new();
    let mut window_start_size = globals.win_sdl.window.size();

    unsafe {
        gl::Viewport(0, 0, window_start_size.0 as i32, window_start_size.1 as i32);
    }
    //// depth only shader
    let depth_only_shader = create_program(
        "./shaders/BasicModelDepthOnly/shader.vert",
        "./shaders/BasicModelDepthOnly/shader.frag",
    )
    .unwrap();


    ////deferred passes shader programs
    let point_lighting_pass = create_program(
        "./shaders/PointLightingPass/shader.vert",
        "./shaders/PointLightingPass/shader.frag",
    )
    .unwrap();
    let directional_lighting_pass = create_program(
        "./shaders/DirectionalLightingPass/shader.vert",
        "./shaders/DirectionalLightingPass/shader.frag",
    )
    .unwrap();
    let spot_lighting_pass = create_program(
        "./shaders/SpotLightingPass/shader.vert",
        "./shaders/SpotLightingPass/shader.frag",
    )
    .unwrap();
    let final_pass = globals.screen_model.shader_program.clone();

    ////lighting pass gbuffer
    let (g_position, g_normal, g_albedo_spec, gbuffer, g_render_buffer) =
        create_gbuffer(window_start_size);
    let (light_texture, light_buffer) = create_framebuffer(window_start_size);

    let mut directional_lights: Vec<DirectionalLight> = Vec::new();
    directional_lights.push(DirectionalLight::new((4096, 4096)));
    directional_lights[0].position = Vector3::new(-2.0, 4.0, -1.0);
    directional_lights[0].direction =
        (Vector3::zero() - directional_lights[0].position).normalize();

    let mut spot_lights: Vec<SpotLight> = Vec::new();
    spot_lights.push(SpotLight::new((4096, 4096), (90.0f32).to_radians()));
    spot_lights[0].position = Vector3::new(-2.0, 0.0, 0.0);
    spot_lights[0].direction = (Vector3::zero() - spot_lights[0].position).normalize();

    let mut point_lights: Vec<PointLight> = Vec::new();

    ////UNUSED CODE
    /*
    //point_lights.push(PointLight::new());
    //point_lights[0].position = Vector3::new(0., 2., 0.);
    */
    
    ////UNUSED CODE
    /*

    let mut portals: Vec<Portal> = Vec::new();

    let portal_shader_program: Program = create_program("./shaders/Portal/shader.vert", "./shaders/Portal/shader.frag").unwrap();
    let mut portal_1_model = Model::from_obj_file("./models/Portal.obj".to_owned());
    portal_1_model.position = Vector3::new(0., 0., 2.);
    portal_1_model.shader_program = portal_shader_program;
    portal_1_model.render_shadows = false;

    let portal_shader_program: Program = create_program("./shaders/Portal/shader.vert", "./shaders/Portal/shader.frag").unwrap();
    let mut portal_2_model = Model::from_obj_file("./models/Portal.obj".to_owned());
    
    portal_2_model.position = Vector3::new(0., 0., -2.);
    portal_2_model.shader_program = portal_shader_program;
    portal_2_model.render_shadows = false;
    globals.models.push(portal_1_model);
    let portal_1 = Portal::new(globals.models.len()-1, portals.len()+1, &mut globals);
    portals.push(portal_1);
    globals.models.push(portal_2_model);
    let portal_2 = Portal::new(globals.models.len()-1, portals.len()-1, &mut globals);
    portals.push(portal_2); */

    let start_time = Instant::now();
    let mut deltatime: f32 = 0.0;

    let mut steal_mouse: bool;
    let mut reset_mouse: bool = false;

    'running: loop {
        let frame_start = Instant::now();
        let size = globals.win_sdl.window.size();
        globals.cam.projection_matrix.aspect = size.0 as f32 / size.1 as f32;
        globals.egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        globals
            .egui_ctx
            .begin_frame(globals.egui_state.input.take());

        if window_start_size != size {
            remake_gbuffer(
                &gbuffer,
                &g_position,
                &g_normal,
                &g_albedo_spec,
                &g_render_buffer,
                size,
            );

            remake_framebuffer(&light_buffer, &light_texture, size);
            window_start_size = size;
        }

        for event in globals.win_sdl.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    ..
                } => {
                    globals.should_grab_mouse = !globals.should_grab_mouse;
                    reset_mouse = true;
                }
                _ => {
                    globals.egui_state.process_input(
                        &globals.win_sdl.window,
                        event,
                        &mut globals.egui_painter,
                    );
                }
            }
        }
        handle_input(&mut globals, deltatime);
        let mut mouse_delta: (f32, f32) = (0.0, 0.0);

        steal_mouse = globals.win_sdl.window.has_input_focus();

        if steal_mouse && globals.should_grab_mouse {
            let mouse_state = globals.win_sdl.event_pump.mouse_state();
            if !reset_mouse {
                mouse_delta = (
                    (mouse_state.x() as f32 - (size.0 / 2) as f32) / size.1 as f32,
                    (mouse_state.y() as f32 - (size.1 / 2) as f32) / size.1 as f32,
                );
            } else {
                reset_mouse = false;
            }
            globals.win_sdl._sdl_context.mouse().warp_mouse_in_window(
                &globals.win_sdl.window,
                (size.0 / 2) as i32,
                (size.1 / 2) as i32,
            );
            globals.win_sdl.window.set_mouse_grab(true);
            globals.win_sdl._sdl_context.mouse().show_cursor(false);
        } else {
            reset_mouse = true;
            globals.win_sdl.window.set_mouse_grab(false);
            globals.win_sdl._sdl_context.mouse().show_cursor(true);
        };

        //println!("{:?}", mouse_delta);

        let mouse_look_delta = (
            -mouse_delta.0
                * globals.cam.projection_matrix.fovy.0
                * globals.mouse_look_sensitivity
                * deltatime
                * 75.0
                * 2.0,
            -mouse_delta.1 * globals.cam.projection_matrix.fovy.0 * size.0 as f32 / size.1 as f32
                * globals.mouse_look_sensitivity
                * deltatime
                * 75.0
                * 2.0,
        );

        globals.cam.camera_rotation += mouse_look_delta.into();

        for directional_light in &mut directional_lights {
            directional_light.render(&mut globals, &depth_only_shader);
        }
        for spot_light in &mut spot_lights {
            spot_light.render(&mut globals, &depth_only_shader);
        }


        ////UNUSED CODE, am still figuring out how to implement portals
        //for i in 0..portals.len() {
        //    let portal = portals.get(i).unwrap();
        //    portal.render(&mut globals, &mut portals[portal.connected_portal_index], 0, 5, i);
        //}

        unsafe {
            gl::Viewport(
                0,
                0,
                globals.win_sdl.window.size().0 as i32,
                globals.win_sdl.window.size().1 as i32,
            );
        }

        gbuffer.bind();
        let projection_matrix: Matrix4<f32> = globals.cam.projection_matrix.into();
        let view_matrix: Matrix4<f32> = globals.cam.view_transform().invert().unwrap();
        draw_scene(&mut globals, projection_matrix, view_matrix);
        gbuffer.unbind();

        light_buffer.bind();

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE);
        }

        for i in 0..point_lights.len() {
            draw_point_lighting_pass(
                &mut globals,
                &point_lighting_pass,
                &g_position,
                &g_normal,
                &g_albedo_spec,
                &point_lights[i],
            );
        }

        for i in 0..directional_lights.len() {
            draw_directional_lighting_pass(
                &mut globals,
                &directional_lighting_pass,
                &g_position,
                &g_normal,
                &g_albedo_spec,
                &directional_lights[i],
            );
        }

        for i in 0..spot_lights.len() {
            draw_spot_lighting_pass(
                &mut globals,
                &spot_lighting_pass,
                &g_position,
                &g_normal,
                &g_albedo_spec,
                &spot_lights[i],
            );
        }

        light_buffer.unbind();

        ////draw final pass
        unsafe {
            gl::Disable(gl::BLEND);
        }

        draw_final_pass(
            &mut globals,
            &final_pass,
            &g_position,
            &g_normal,
            &g_albedo_spec,
            &light_texture,
        );

        draw_scene_light_points(&mut globals, &point_lights, &spot_lights);

        draw_ui(
            &mut globals,
            deltatime,
            &mut point_lights,
            &mut directional_lights,
            &mut spot_lights,
        );

        globals.win_sdl.window.gl_swap_window();

        deltatime = frame_start.elapsed().as_secs_f32();
    }
}

pub fn draw_scene(
    globals: &mut Globals,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
) {
    unsafe {
        gl::ClearColor(0. / 255., 0. / 255., 0. / 255., 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }


    for model in &mut globals.models {
        let screen_size: Vector2<f32> = Vector2::new(globals.win_sdl.window.size().0 as f32,globals.win_sdl.window.size().1 as f32);
        if model.render_shadows {
            unsafe {
                gl::CullFace(gl::BACK);
            }
            model.start_render();
            
            model.render(screen_size, view_matrix, projection_matrix);
        } else {
            unsafe {
                gl::CullFace(gl::BACK);
            }
            model.start_render();
            model.render_fullbright(screen_size, view_matrix, projection_matrix);
        }
        
    }
}

pub fn draw_scene_shadows(
    globals: &mut Globals,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
) {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }
    unsafe {
        gl::ClearColor(0. / 255., 0. / 255., 0. / 255., 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    for model in &mut globals.models {
        unsafe {
            gl::CullFace(gl::FRONT);
        }
        model.start_render();
        model.render_fullbright(Vector2::zero(), view_matrix, projection_matrix);
        unsafe {
            gl::CullFace(gl::CULL_FACE);
        }
    }
}

pub fn draw_scene_custom_shader_program(
    globals: &mut Globals,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    shader_program: &Program,
    is_render_shadows: bool,
) {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }
    unsafe {
        gl::ClearColor(0. / 255., 0. / 255., 0. / 255., 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }


    for model in &mut globals.models {
        if (is_render_shadows && model.render_shadows) || !is_render_shadows {
            unsafe {
                gl::CullFace(gl::FRONT);
            }
            model.start_render_custom_shader_program(shader_program);
            model.shader_program.set_float("uSize.x", globals.win_sdl.window.size().0 as f32);
            model.shader_program.set_float("uSize.y", globals.win_sdl.window.size().1 as f32);
            model.render_custom_shader_program(view_matrix, projection_matrix, shader_program);
            unsafe {
                gl::CullFace(gl::CULL_FACE);
            }
        }
        
    }
}

pub fn draw_point_lighting_pass(
    globals: &mut Globals,
    shader_program: &Program,
    g_position: &Texture,
    g_normal: &Texture,
    g_albedo_spec: &Texture,
    point_light: &PointLight,
) {
    shader_program.set();
    globals.screen_model.vbo.bind();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
    }
    g_position.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE1);
    }
    g_normal.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE2);
    }
    g_albedo_spec.bind_texture();
    shader_program.set_int("gPosition", 0);
    shader_program.set_int("gNormal", 1);
    shader_program.set_int("gAlbedoSpec", 2);
    shader_program.set_vector3("viewPos", globals.cam.position);

    globals.screen_model.vao.bind();
    globals.screen_model.ibo.bind();
    shader_program.set_point_light_info("light", point_light.info, point_light.position);

    globals.screen_model.render_fullbright(Vector2::zero(),
        globals.cam.view_transform().invert().unwrap(),
        globals.cam.projection_matrix.into(),
    );
}

pub fn draw_directional_lighting_pass(
    globals: &mut Globals,
    shader_program: &Program,
    g_position: &Texture,
    g_normal: &Texture,
    g_albedo_spec: &Texture,
    directional_light: &DirectionalLight,
) {
    shader_program.set();
    globals.screen_model.vbo.bind();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
    }
    g_position.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE1);
    }
    g_normal.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE2);
    }
    g_albedo_spec.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE3);
    }
    directional_light.shadow_texture.bind_texture();

    shader_program.set_int("gPosition", 0);
    shader_program.set_int("gNormal", 1);
    shader_program.set_int("gAlbedoSpec", 2);
    shader_program.set_int("shadowMap", 3);
    shader_program.set_vector3("viewPos", globals.cam.position);

    globals.screen_model.vao.bind();
    globals.screen_model.ibo.bind();
    shader_program.set_vector3("light.Position", directional_light.position);
    shader_program.set_vector3("light.Direction", directional_light.direction);
    shader_program.set_matrix4_float(
        "lightSpaceMatrix",
        directional_light.light_projection * directional_light.light_view,
    );
    shader_program.set_vector3("light.Color", directional_light.info.color);

    globals.screen_model.render_fullbright(Vector2::zero(),
        globals.cam.view_transform().invert().unwrap(),
        globals.cam.projection_matrix.into(),
    );
}

pub fn draw_spot_lighting_pass(
    globals: &mut Globals,
    shader_program: &Program,
    g_position: &Texture,
    g_normal: &Texture,
    g_albedo_spec: &Texture,
    spot_light: &SpotLight,
) {
    shader_program.set();
    globals.screen_model.vbo.bind();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
    }
    g_position.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE1);
    }
    g_normal.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE2);
    }
    g_albedo_spec.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE3);
    }
    spot_light.shadow_texture.bind_texture();

    shader_program.set_int("gPosition", 0);
    shader_program.set_int("gNormal", 1);
    shader_program.set_int("gAlbedoSpec", 2);
    shader_program.set_int("shadowMap", 3);
    shader_program.set_vector3("viewPos", globals.cam.position);

    globals.screen_model.vao.bind();
    globals.screen_model.ibo.bind();
    shader_program.set_vector3("light.Position", spot_light.position);
    shader_program.set_vector3("light.Direction", spot_light.direction);
    shader_program.set_matrix4_float(
        "lightSpaceMatrix",
        spot_light.light_projection * spot_light.light_view,
    );
    shader_program.set_vector3("light.Color", spot_light.info.color);
    shader_program.set_float("light.Radius", spot_light.info.radius);
    shader_program.set_float("light.Fov", spot_light.horizontal_fov);

    globals.screen_model.render_fullbright(Vector2::zero(),
        globals.cam.view_transform().invert().unwrap(),
        globals.cam.projection_matrix.into(),
    );
}

pub fn draw_final_pass(
    globals: &mut Globals,
    shader_program: &Program,
    g_position: &Texture,
    g_normal: &Texture,
    g_albedo_spec: &Texture,
    g_lighting: &Texture,
) {
    shader_program.set();
    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }
    globals.screen_model.vbo.bind();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
    }
    g_position.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE1);
    }
    g_normal.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE2);
    }
    g_albedo_spec.bind_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE3);
    }
    g_lighting.bind_texture();
    shader_program.set_int("gPosition", 0);
    shader_program.set_int("gNormal", 1);
    shader_program.set_int("gAlbedoSpec", 2);
    globals.screen_model.shader_program.set_int("gLighting", 3);
    shader_program.set_vector3("viewPos", globals.cam.position);

    globals.screen_model.vao.bind();
    globals.screen_model.ibo.bind();
    globals.screen_model.render_fullbright(Vector2::zero(),
        globals.cam.view_transform().invert().unwrap(),
        globals.cam.projection_matrix.into(),
    );
}

pub fn draw_scene_light_points(
    globals: &mut Globals,
    point_lights: &Vec<PointLight>,
    spot_lights: &Vec<SpotLight>,
) {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }

    for point_light in point_lights {
        globals.light_model.position = point_light.position;
        globals.light_model.start_render();
        globals.light_model.render_fullbright(Vector2::zero(),
            globals.cam.view_transform().invert().unwrap(),
            globals.cam.projection_matrix.into(),
        );
    }
    for spot_light in spot_lights {
        globals.light_model.position = spot_light.position;
        globals.light_model.rotation = Quaternion::look_at(spot_light.direction, Vector3::unit_y());
        globals.light_model.start_render();
        globals.light_model.render_fullbright(Vector2::zero(),
            globals.cam.view_transform().invert().unwrap(),
            globals.cam.projection_matrix.into(),
        );
    }
}

pub fn draw_ui(
    globals: &mut Globals,
    deltatime: f32,
    point_lights: &mut Vec<PointLight>,
    directional_lights: &mut Vec<DirectionalLight>,
    spot_lights: &mut Vec<SpotLight>,
) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    let egui_debug_ui = egui::Window::new("Debug Values");
    let fps = 1.0 / deltatime;
    egui_debug_ui.show(&globals.egui_ctx, |ui| {
        ui.label("FPS: ".to_owned() + &fps.to_string());
        ui.separator();
        ui.label("Movement Speed");
        ui.add(
            egui::DragValue::new(&mut globals.movement_speed)
                .suffix(" m")
                .speed(0.01),
        );
        ui.label("Look Sensitivity");
        ui.add(egui::DragValue::new(&mut globals.look_sensitivity).speed(0.01));
        ui.label("Mouse Look Sensitivity");
        ui.add(egui::DragValue::new(&mut globals.mouse_look_sensitivity).speed(0.01));
        ui.separator();
        egui::ScrollArea::vertical()
            .id_source("explorer_scroll_area")
            .show(ui, |ui| {
                ui.label("Models");
                egui::ScrollArea::vertical()
                    .id_source("model_scroll_area")
                    .show(ui, |ui| {
                        for model in &mut globals.models {
                            ui.add(model);
                        }
                    });
                ui.separator();
                ui.label("Point Lights");

                egui::ScrollArea::vertical()
                    .id_source("point_light_scroll_area")
                    .show(ui, |ui| {
                        for point_light in point_lights.iter_mut() {
                            ui.add(point_light);
                        }
                    });
                ui.separator();
                ui.label("Directional Lights");

                egui::ScrollArea::vertical()
                    .id_source("directional_light_scroll_area")
                    .show(ui, |ui| {
                        for directional_light in directional_lights.iter_mut() {
                            ui.add(directional_light);
                        }
                    });
                egui::ScrollArea::vertical()
                    .id_source("spot_light_scroll_area")
                    .show(ui, |ui| {
                        for spot_light in spot_lights.iter_mut() {
                            ui.add(spot_light);
                        }
                    });
            });
    });

    unsafe {
        gl::Disable(gl::DEPTH_TEST);
    }

    let FullOutput {
        platform_output,
        textures_delta,
        shapes,
        pixels_per_point,
        viewport_output: _,
    } = globals.egui_ctx.end_frame();

    // Process ouput
    globals
        .egui_state
        .process_output(&globals.win_sdl.window, &platform_output);

    let paint_jobs = globals.egui_ctx.tessellate(shapes, pixels_per_point);
    globals
        .egui_painter
        .paint_jobs(None, textures_delta, paint_jobs);
}

pub fn remake_framebuffer(framebuffer: &FrameBuffer, texture: &Texture, size: (u32, u32)) {
    framebuffer.bind();
    texture.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            null(),
        );
    }
}

pub fn create_framebuffer(size: (u32, u32)) -> (Texture, FrameBuffer) {
    let framebuffer = FrameBuffer::new();
    framebuffer.bind();
    let texture = Texture::new();
    texture.make_empty(size).unwrap();
    framebuffer.load(&texture).unwrap();
    return (texture, framebuffer);
}

pub fn remake_gbuffer(
    gbuffer: &FrameBuffer,
    g_position: &Texture,
    g_normal: &Texture,
    g_albedo_spec: &Texture,
    g_render_buffer: &RenderBuffer,
    size: (u32, u32),
) {
    gbuffer.bind();
    g_position.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            null(),
        );
    };
    g_normal.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            null(),
        );
    };
    g_albedo_spec.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            null(),
        );
    };
    g_render_buffer.bind();
    unsafe {
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            size.0 as i32,
            size.1 as i32,
        );
    }
}

pub fn create_gbuffer(size: (u32, u32)) -> (Texture, Texture, Texture, FrameBuffer, RenderBuffer) {
    let gbuffer = FrameBuffer::new();
    gbuffer.bind();
    let g_position = Texture::new();
    g_position.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            null(),
        );
    };
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    };
    unsafe {
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            g_position.id,
            0,
        );
    };
    let g_normal = Texture::new();
    g_normal.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            null(),
        );
    };
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    };
    unsafe {
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT1,
            gl::TEXTURE_2D,
            g_normal.id,
            0,
        );
    };
    let g_albedo_spec = Texture::new();
    g_albedo_spec.bind();
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            null(),
        );
    };
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    };
    unsafe {
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT2,
            gl::TEXTURE_2D,
            g_albedo_spec.id,
            0,
        );
    };

    unsafe {
        gl::DrawBuffers(
            3,
            [
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
            ]
            .as_ptr() as *const u32,
        );
    }

    let g_render_buffer = RenderBuffer::new();
    g_render_buffer.load(&gbuffer, size).unwrap();

    return (
        g_position,
        g_normal,
        g_albedo_spec,
        gbuffer,
        g_render_buffer,
    );
}

pub fn create_framebuffer_depthbuffer(size: (u32, u32)) -> (Texture, FrameBuffer) {
    let framebuffer = FrameBuffer::new();
    let texture = Texture::new();
    texture.make_empty_depth_buffer(size).unwrap();
    framebuffer.load_depth_texture(&texture).unwrap();

    return (texture, framebuffer);
}

pub fn create_framebuffer_depth_cubemap(size: (u32, u32)) -> (Cubemap, FrameBuffer) {
    let framebuffer = FrameBuffer::new();
    let cubemap = Cubemap::new();
    cubemap.make_empty_depth_buffer(size).unwrap();
    framebuffer.load_depth_cubemap(&cubemap).unwrap();

    return (cubemap, framebuffer);
}

pub fn handle_input(globals: &mut Globals, deltatime: f32) {
    let keyboard_state = globals.win_sdl.event_pump.keyboard_state();
    let mut movement_position: Vector3<f32> = Vector3::zero();
    let mut movement_rotation: Vector2<f32> = Vector2::zero();
    if keyboard_state.is_scancode_pressed(Scancode::W) {
        movement_position.z += -globals.movement_speed * deltatime;
    } else if keyboard_state.is_scancode_pressed(Scancode::S) {
        movement_position.z += globals.movement_speed * deltatime;
    }
    if keyboard_state.is_scancode_pressed(Scancode::A) {
        movement_position.x += -globals.movement_speed * deltatime;
    } else if keyboard_state.is_scancode_pressed(Scancode::D) {
        movement_position.x += globals.movement_speed * deltatime;
    }
    if keyboard_state.is_scancode_pressed(Scancode::Q) {
        movement_position.y += -globals.movement_speed * deltatime;
    } else if keyboard_state.is_scancode_pressed(Scancode::E) {
        movement_position.y += globals.movement_speed * deltatime;
    }

    if keyboard_state.is_scancode_pressed(Scancode::Up) {
        movement_rotation.y += f32::consts::PI * deltatime * globals.look_sensitivity;
    } else if keyboard_state.is_scancode_pressed(Scancode::Down) {
        movement_rotation.y += -f32::consts::PI * deltatime * globals.look_sensitivity;
    }
    if keyboard_state.is_scancode_pressed(Scancode::Left) {
        movement_rotation.x += f32::consts::PI * deltatime * globals.look_sensitivity;
    } else if keyboard_state.is_scancode_pressed(Scancode::Right) {
        movement_rotation.x += -f32::consts::PI * deltatime * globals.look_sensitivity;
    }

    let new_rotation: Quaternion<f32> =
        Quaternion::from_angle_y(Rad(globals.cam.camera_rotation.x));
    movement_position = new_rotation.rotate_vector(movement_position);
    globals.cam.camera_rotation += movement_rotation;
    globals.cam.position += movement_position;
}
