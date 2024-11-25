use std::{os::raw::c_void, path::Path, ptr::null};

use cgmath::{Vector2, Vector3};
use image::{EncodableLayout, ImageError};

pub struct VertexBuffer {
    id: u32,
}
impl VertexBuffer {
    pub fn data(&self, data: &Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)>) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<f32>() * 11) as isize,
                data.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
            /*let pointer = data.as_ptr() as *mut gl::types::GLvoid;
            println!("{:?}", pointer);
            for i in 0..data.len() {
                for j in 0..11 {
                    let new_pointer = pointer.add(i * 11 * std::mem::size_of::<f32>() + j * std::mem::size_of::<f32>());
                    println!("{:?}", *(new_pointer as *const f32) as f32);
                    new_pointer.drop_in_place();
                }
            }
            pointer.drop_in_place();*/
        }
    }
}
impl Drop for VertexBuffer {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl VertexBuffer {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        };
        VertexBuffer { id }
    }

    pub fn set(&self, data: &Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)>) {
        self.bind();
        self.data(data);
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct IndexBuffer {
    id: u32,
}
impl IndexBuffer {
    pub fn data(&self, data: &Vec<u32>) {
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<f32>()) as isize,
                data.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            )
        }
    }
}
impl Drop for IndexBuffer {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl IndexBuffer {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        };
        return IndexBuffer { id };
    }

    pub fn set(&self, data: &Vec<u32>) {
        self.bind();
        self.data(data);
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct VertexArrayBuffer {
    id: u32,
}
impl Drop for VertexArrayBuffer {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl VertexArrayBuffer {
    fn setup(&self) {
        unsafe {
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (11 * std::mem::size_of::<f32>()) as i32,
                0 as *const c_void,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (11 * std::mem::size_of::<f32>()) as i32,
                ((3 * std::mem::size_of::<f32>()) as i32) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                (11 * std::mem::size_of::<f32>()) as i32,
                ((6 * std::mem::size_of::<f32>()) as i32) as *const c_void,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                3,
                2,
                gl::FLOAT,
                gl::FALSE,
                (11 * std::mem::size_of::<f32>()) as i32,
                ((9 * std::mem::size_of::<f32>()) as i32) as *const c_void,
            );
            gl::EnableVertexAttribArray(3);
        }
    }
}
impl VertexArrayBuffer {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        };
        return VertexArrayBuffer { id };
    }

    pub fn set(&self) {
        self.bind();
        self.setup();
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub trait ModelTexture {
    fn bind_texture(&self);
}
#[derive(Debug, Clone)]
pub struct Texture {
    pub id: u32,
}
impl Texture {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        Self { id }
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl Texture {
    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id) }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) }
    }
    pub fn delete(&self) {
        unsafe {
            gl::DeleteTextures(1, [self.id].as_ptr());
        }
    }
    pub fn load(&self, path: &Path) -> Result<(), ImageError> {
        self.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::open(path)?.flipv().into_rgba8();
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_bytes().as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Ok(())
    }
    pub fn make_empty(&self, size: (u32, u32)) -> Result<(), ImageError> {
        self.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
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
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Ok(())
    }
    pub fn make_empty_depth_buffer(&self, size: (u32, u32)) -> Result<(), ImageError> {
        self.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as i32,
                size.0 as i32,
                size.1 as i32,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                null(),
            );
        }
        Ok(())
    }
}
impl ModelTexture for Texture {
    fn bind_texture(&self) {
        self.bind();
    }
}

#[derive(Debug, Clone)]
pub struct Cubemap {
    pub id: u32,
}
impl Cubemap {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        Self { id }
    }
}
impl Drop for Cubemap {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl Cubemap {
    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.id) }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0) }
    }
    pub fn delete(&self) {
        unsafe {
            gl::DeleteTextures(1, [self.id].as_ptr());
        }
    }
    //pub fn load(&self, path: &Path) -> Result<(), ImageError> {
    //    self.bind();
    //    unsafe {
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    //    }
    //    let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
    //        image::open(path)?.flipv().into_rgba8();
    //    unsafe {
    //        gl::TexImage2D(
    //            gl::TEXTURE_2D,
    //            0,
    //            gl::RGBA as i32,
    //            img.width() as i32,
    //            img.height() as i32,
    //            0,
    //            gl::RGBA,
    //            gl::UNSIGNED_BYTE,
    //            img.as_bytes().as_ptr() as *const _,
    //        );
    //        gl::GenerateMipmap(gl::TEXTURE_2D);
    //    }
    //    Ok(())
    //}
    //pub fn make_empty(&self, size: (u32, u32)) -> Result<(), ImageError> {
    //    self.bind();
    //    unsafe {
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    //        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    //    }
    //    unsafe {
    //        gl::TexImage2D(
    //            gl::TEXTURE_2D,
    //            0,
    //            gl::RGBA as i32,
    //            size.0 as i32,
    //            size.1 as i32,
    //            0,
    //            gl::RGBA,
    //            gl::UNSIGNED_BYTE,
    //            null(),
    //        );
    //        gl::GenerateMipmap(gl::TEXTURE_2D);
    //    }
    //    Ok(())
    //}
    pub fn make_empty_depth_buffer(&self, size: (u32, u32)) -> Result<(), ImageError> {
        self.bind();
        for i in 0..6 {
            unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::DEPTH_COMPONENT as i32,
                    size.0 as i32,
                    size.1 as i32,
                    0,
                    gl::DEPTH_COMPONENT,
                    gl::FLOAT,
                    null(),
                );
            }
        }
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        Ok(())
    }
}
impl ModelTexture for Cubemap {
    fn bind_texture(&self) {
        self.bind();
    }
}

pub struct FrameBuffer {
    pub id: u32,
}

impl FrameBuffer {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }
        Self { id }
    }
}
impl Drop for FrameBuffer {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl FrameBuffer {
    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.id) };
    }

    pub fn unbind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };
    }
    pub fn delete(&self) {
        unsafe {
            gl::DeleteFramebuffers(1, [self.id].as_ptr());
        }
    }
    pub fn load(&self, texture: &Texture) -> Result<(), String> {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.id,
                0,
            );
        }

        Ok(())
    }
    pub fn load_depth_texture(&self, texture: &Texture) -> Result<(), String> {
        texture.bind();
        self.bind();
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                texture.id,
                0,
            );
        };
        unsafe {
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
        }
        self.unbind();

        Ok(())
    }
    pub fn load_depth_cubemap(&self, cubemap: &Cubemap) -> Result<(), String> {
        cubemap.bind();
        self.bind();
        unsafe {
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, cubemap.id, 0);
        };
        unsafe {
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
        }
        self.unbind();

        Ok(())
    }
}
impl ModelTexture for FrameBuffer {
    fn bind_texture(&self) {
        self.bind();
    }
}

pub struct RenderBuffer {
    pub id: u32,
}
impl RenderBuffer {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut id);
        }
        Self { id }
    }
}
impl Drop for RenderBuffer {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}
impl RenderBuffer {
    pub fn bind(&self) {
        unsafe { gl::BindRenderbuffer(gl::RENDERBUFFER, self.id) };
    }

    pub fn unbind(&self) {
        unsafe { gl::BindRenderbuffer(gl::RENDERBUFFER, 0) };
    }
    pub fn delete(&self) {
        unsafe {
            gl::DeleteRenderbuffers(1, [self.id].as_ptr());
        }
    }
    pub fn load(&self, framebuffer: &FrameBuffer, size: (u32, u32)) -> Result<(), String> {
        framebuffer.bind();
        self.bind();
        unsafe {
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                size.0 as i32,
                size.1 as i32,
            );
        }
        unsafe {
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                self.id,
            );
        }
        if unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) } != gl::FRAMEBUFFER_COMPLETE {
            return Err("ERROR::FRAMEBUFFER:: Framebuffer is not complete!".to_owned());
        }
        Ok(())
    }
}
impl ModelTexture for RenderBuffer {
    fn bind_texture(&self) {
        self.bind();
    }
}
