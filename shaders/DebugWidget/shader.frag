#version 330 core
in vec3 VertColor;
in vec2 TexCoord;
in vec3 VertPos;

out vec4 FragColor;

uniform sampler2D texture0;
uniform vec3 color;

void main()
{
    FragColor = vec4(color, 1.0); // set all 4 vector values to 1.0
}