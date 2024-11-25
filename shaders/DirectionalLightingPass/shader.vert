#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec3 aNormal;
layout (location = 3) in vec2 aTexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;


out vec3 VertColor;
out vec2 TexCoords;
out vec3 VertPos;
out vec3 Normal;

out vec3 FragPos;

void main()
{
    gl_Position = vec4(aPos, 1.0);
    FragPos = vec3(vec4(aPos, 1.0));

    VertColor = aColor;
    VertPos = aPos;
    TexCoords = aTexCoord;
    Normal = aNormal;
}