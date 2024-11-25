use cgmath::{Vector2, Vector3};

use crate::{material_structs::MaterialInfo, models::Model};

pub struct F322DVectorTo3DModel {
    pub model_index: usize,
    pub values: Vec<Vec<f32>>,
    pub scale: f32,
}

impl F322DVectorTo3DModel {
    pub fn new(model_index: usize, width: usize, height: usize, scale: f32) -> Self {
        let mut values: Vec<Vec<f32>> = Vec::new();
        for _i in 0..width {
            let mut lower_value: Vec<f32> = Vec::new();
            for _j in 0..height {
                let value: f32 = 0.;
                lower_value.push(value);
            }
            values.push(lower_value);
        }

        return Self {
            model_index,
            values,
            scale,
        };
    }
    pub fn create_geometry(&self, models: &mut Vec<Model>) {
        let model = &mut models[self.model_index];
        let mut vertices: Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>)> =
            Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for i in 0..self.values.len() {
            for j in 0..self.values[i].len() {
                let vertice: (Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector2<f32>) = (
                    Vector3::new(
                        i as f32 * self.scale,
                        self.values[i][j] * self.scale,
                        j as f32 * self.scale,
                    ),
                    Vector3::new(0.529, 0.808, 0.922),
                    Vector3::unit_y(),
                    Vector2::new(i as f32, j as f32),
                );
                vertices.push(vertice);
            }
        }
        for i in 0..self.values.len() - 1 {
            for j in 0..self.values[i].len() - 1 {
                indices.push((i + j * self.values[i].len()) as u32);
                indices.push(((i + 1) + (j) * self.values[i + 1].len()) as u32);
                indices.push(((i) + (j + 1) * self.values[i].len()) as u32);

                indices.push(((i) + (j + 1) * self.values[i].len()) as u32);
                indices.push(((i + 1) + (j) * self.values[i + 1].len()) as u32);
                indices.push(((i + 1) + (j + 1) * self.values[i + 1].len()) as u32);
            }
        }
        model.vertices = vertices;
        model.indices = indices;
        model.material_info = MaterialInfo {
            ambient: Vector3::new(1., 1., 1.),
            diffuse: Vector3::new(1., 1., 1.),
            specular: 0.,
            shininess: 32.,
        };
        model.start();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new() {
        let width = 2;
        let height = 2;
        let model_index = 0; //this doesn't matter here
        let tested_thing = F322DVectorTo3DModel::new(model_index, width, height, 1.);
        if tested_thing.values.len() != width {
            //if vector width is not width
            panic!();
        }
        if tested_thing.values[0].len() != height {
            //if vector height is not height
            panic!();
        }
        for i in 0..width {
            for j in 0..height {
                let value = tested_thing.values[i][j];
                if value != 0. {
                    //if not all values are 0
                    panic!();
                }
            }
        }
    }
}
