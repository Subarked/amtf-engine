use cgmath::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct MaterialInfo {
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: f32,

    pub shininess: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLightInfo {
    pub color: Vector3<f32>,
}
#[derive(Debug, Clone, Copy)]
pub struct PointLightInfo {
    pub color: Vector3<f32>,

    pub radius: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct SpotLightInfo {
    pub color: Vector3<f32>,

    pub radius: f32,
}
