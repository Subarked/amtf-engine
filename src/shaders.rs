use std::{
    ffi::{CStr, CString},
    fs,
    ptr::{null, null_mut},
};

use cgmath::{Matrix, Matrix4, Vector2, Vector3};
use gl::types::GLenum;

use crate::material_structs::{MaterialInfo, PointLightInfo, SpotLightInfo};

pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn from_source(source_a: &str, kind: GLenum) -> Result<Self, String> {
        let source: &CStr = &CString::new(source_a).unwrap();
        let id = unsafe { gl::CreateShader(kind) };
        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), null());
        }
        unsafe {
            gl::CompileShader(id);
        }
        let mut success: i32 = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }
        if success == 0 {
            let mut len: i32 = 0;
            unsafe { gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len) }

            let error: CString = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetShaderInfoLog(id, len, null_mut(), error.as_ptr() as *mut i8);
            }

            return Err(error.to_string_lossy().into_owned());
        }
        Ok(Shader { id })
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}
#[derive(Debug, Clone)]
pub struct Program {
    id: u32,
}

impl Program {
    fn from_shaders(shaders: &[Shader]) -> Result<Self, String> {
        let id = unsafe { gl::CreateProgram() };
        for shader in shaders {
            unsafe {
                gl::AttachShader(id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(id);
        }
        let mut success: i32 = 1;
        unsafe {
            gl::GetProgramiv(id, gl::COMPILE_STATUS, &mut success);
        }
        if success == 0 {
            let mut len: i32 = 0;
            unsafe { gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len) }

            let error: CString = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetProgramInfoLog(id, len, null_mut(), error.as_ptr() as *mut i8);
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl::DetachShader(id, shader.id());
            }
        }
        Ok(Program { id })
    }

    pub fn set(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
    pub fn set_bool(&self, name_a: &str, value: bool) {
        let name: &CStr = &CString::new(name_a).unwrap();
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
        }
    }
    pub fn set_int(&self, name_a: &str, value: i32) {
        let name: &CStr = &CString::new(name_a).unwrap();
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
        }
    }
    pub fn set_float(&self, name_a: &str, value: f32) {
        let name: &CStr = &CString::new(name_a).unwrap();
        unsafe {
            gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value as f32);
        }
    }
    pub fn set_matrix4_float(&self, name_a: &str, value: Matrix4<f32>) {
        let name: &CStr = &CString::new(name_a).unwrap();
        unsafe {
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.id, name.as_ptr()),
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }
    pub fn set_vector3(&self, name_a: &str, value: Vector3<f32>) {
        let name: &CStr = &CString::new(name_a).unwrap();
        unsafe {
            gl::Uniform3f(
                gl::GetUniformLocation(self.id, name.as_ptr()),
                value.x,
                value.y,
                value.z,
            );
        }
    }
    pub fn set_vector2(&self, name_a: &str, value: Vector2<f32>) {
        let name: &CStr = &CString::new(name_a).unwrap();
        unsafe {
            gl::Uniform2f(
                gl::GetUniformLocation(self.id, name.as_ptr()),
                value.x,
                value.y,
            );
        }
    }
    pub fn set_material_info(&self, name_a: &str, value: MaterialInfo) {
        self.set_vector3(&(name_a.to_owned() + ".ambient"), value.ambient);
        self.set_vector3(&(name_a.to_owned() + ".diffuse"), value.diffuse);
        self.set_float(&(name_a.to_owned() + ".specular"), value.specular);
        self.set_float(&(name_a.to_owned() + ".shininess"), value.shininess);
    }
    pub fn set_point_light_info(
        &self,
        name_a: &str,
        value: PointLightInfo,
        position: Vector3<f32>,
    ) {
        self.set_vector3(&(name_a.to_owned() + ".Color"), value.color);

        self.set_float(&(name_a.to_owned() + ".Radius"), value.radius);

        self.set_vector3(&(name_a.to_owned() + ".Position"), position);
    }
    pub fn set_spot_light_info(&self, name_a: &str, value: SpotLightInfo, position: Vector3<f32>) {
        self.set_vector3(&(name_a.to_owned() + ".Color"), value.color);

        self.set_float(&(name_a.to_owned() + ".Radius"), value.radius);

        self.set_vector3(&(name_a.to_owned() + ".Position"), position);
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn create_program(
    vert_shader_path: &str,
    frag_shader_path: &str,
) -> Result<Program, &'static str> {
    let vert_shader_text = fs::read_to_string(vert_shader_path).unwrap();
    let frag_shader_text = fs::read_to_string(frag_shader_path).unwrap();

    let vert_shader = Shader::from_source(&vert_shader_text, gl::VERTEX_SHADER).unwrap();
    let frag_shader = Shader::from_source(&frag_shader_text, gl::FRAGMENT_SHADER).unwrap();
    let program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
    Ok(program)
}

pub fn create_program_with_geometry_shader(
    vert_shader_path: &str,
    frag_shader_path: &str,
    geom_shader_path: &str,
) -> Result<Program, &'static str> {
    let vert_shader_text = fs::read_to_string(vert_shader_path).unwrap();
    let frag_shader_text = fs::read_to_string(frag_shader_path).unwrap();
    let geom_shader_text = fs::read_to_string(geom_shader_path).unwrap();

    let vert_shader = Shader::from_source(&vert_shader_text, gl::VERTEX_SHADER).unwrap();
    let frag_shader = Shader::from_source(&frag_shader_text, gl::FRAGMENT_SHADER).unwrap();
    let geom_shader = Shader::from_source(&geom_shader_text, gl::GEOMETRY_SHADER).unwrap();
    let program = Program::from_shaders(&[vert_shader, frag_shader, geom_shader]).unwrap();
    Ok(program)
}
