#version 330 core
out vec4 FragColor;

  
in vec2 TexCoords;


uniform sampler2D gPosition;
uniform sampler2D gNormal;
uniform sampler2D gAlbedoSpec;
uniform sampler2D gLighting;
uniform sampler2D shadowMap;

struct Light {
    vec3 Position;
    vec3 Color;
    vec3 Direction;

    float Fov;
    float Radius;
};

uniform Light light;
uniform vec3 viewPos;
uniform mat4 lightSpaceMatrix;


float ShadowCalculation(vec4 fragPosLightSpace)
{
    // perform perspective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;
    // get closest depth value from light's perspective (using [0,1] range fragPosLight as coords)
    float closestDepth = texture(shadowMap, projCoords.xy).r; 
    // get depth of current fragment from light's perspective
    float currentDepth = projCoords.z;
    // check whether current frag pos is in shadow
    float shadow = currentDepth > closestDepth  ? 1.0 : 0.0;

    return shadow;
}  

void main()
{             
    // retrieve data from G-buffer
    vec3 FragPos = texture(gPosition, TexCoords).rgb;
    vec3 Normal = texture(gNormal, TexCoords).rgb;
    vec3 Albedo = texture(gAlbedoSpec, TexCoords).rgb;
    float Specular = texture(gAlbedoSpec, TexCoords).a;
    vec4 FragPosLightSpace = lightSpaceMatrix * vec4(FragPos,1.0);

    vec3 lightDir = normalize(light.Position - FragPos);
    float theta = dot(lightDir, normalize(-light.Direction));
    float dst = acos(theta)*1/(light.Fov/2);
    float radius = cos(light.Fov / 2);
    float att = clamp(1-dst*dst/(1*1),0,1);
    att *= att;
    if (theta > cos(light.Fov / 2)) {
        // then calculate lighting as usual
        vec3 viewDir = normalize(viewPos - FragPos);
        // diffuse
        vec3 diffuse = max(dot(Normal, lightDir), 0.0) * light.Color;
        // specular
        vec3 halfwayDir = normalize(lightDir + viewDir);  
        float spec = pow(max(dot(Normal, halfwayDir), 0.0), 16.0);
        vec3 specular = light.Color * spec * Specular;
        // attenuation
        float distance = length(light.Position - FragPos);
        float attenuation = clamp(1-distance*distance/(light.Radius*light.Radius), 0.0, 1.0);
        attenuation *= attenuation;
        diffuse *= attenuation;
        specular *= attenuation;
        diffuse *= att;
        specular *= att;
        // calculate shadow
        float shadow = ShadowCalculation(FragPosLightSpace);       
        vec3 lighting = ((1.0 - shadow) * (diffuse + specular)); 

        FragColor = vec4(lighting.rgb, 1.0);
    } else {
        FragColor = vec4(vec3(0), 1.0);
    }
    
} 