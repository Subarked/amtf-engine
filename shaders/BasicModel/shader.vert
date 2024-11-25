#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec3 aNormal;
layout (location = 3) in vec2 aTexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

uniform mat4 light_projection;
uniform mat4 light_view;

out vec3 VertColor;
out vec2 TexCoords;
out vec3 VertPos;
out vec3 Normal;

out vec3 FragPos;
out vec4 FragPosLightSpace;

void main()
{
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    FragPos = vec3(model * vec4(aPos, 1.0));
    FragPosLightSpace = light_projection * light_view * vec4(FragPos,1.0);

    VertColor = aColor;
    VertPos = aPos;
    TexCoords = aTexCoord;
    Normal = mat3(transpose(inverse(model))) * aNormal;
}