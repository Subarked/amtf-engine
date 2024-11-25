#version 330 core
in vec3 VertColor;
in vec2 TexCoords;
in vec3 VertPos;
in vec3 Normal;

in vec3 FragPos;

uniform vec3 viewPos;

uniform float uWidth;
uniform float uHeight;

layout (location = 0) out vec3 gPosition;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec4 gAlbedoSpec;

uniform sampler2D texture0;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    float specular;
    float shininess;
}; 

uniform Material material;

void main()
{
    vec3 normal = normalize(Normal);
    gPosition = FragPos;
    gNormal = normal;
    gAlbedoSpec.rgb = texture(texture0, TexCoords).rgb;
    gAlbedoSpec.a = material.specular;
        
    //float debug = (norm.x > 0.5) ? 1 : 0;
    //FragColor = vec4(debug,debug,debug,1.0);
}