#version 450

layout(location = 0) in vec3 v_WorldPos;
layout(location = 1) in vec3 v_Normal;
layout(location = 2) in vec2 v_Uv;
layout(location = 3) in vec3 WorldPosition;

layout(location = 0) out vec4 FragColor;

const vec3 LightPos = vec3(0.0, 2.0, 0.0);
const vec4 LightColor = vec4(1.0, 1.0, 1.0, 1.0);

layout(set = 2, binding = 0) uniform vec4 AmbientColor;    // Ambient light
layout(set = 2, binding = 1) uniform vec4 DiffuseColor;    // Diffuse color
layout(set = 2, binding = 2) uniform vec4 SpecularColor;   // Specular color
layout(set = 2, binding = 3) uniform paddedShininess {
    float Shininess;
    vec3 _padding1;
};      // Shininess for specular highlight
layout(set = 2, binding = 4) uniform paddedMode {
    int mode;
    vec3 _padding2;
};


vec4 phongShading() {
    vec3 norm = normalize(v_Normal);
    vec3 lightDir = normalize(LightPos - v_WorldPos);
    float diff = max(dot(norm, lightDir), 0.0);

    vec3 viewDir = normalize(WorldPosition - v_WorldPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), Shininess);

    vec4 ambient = AmbientColor;
    vec4 diffuse = diff * DiffuseColor * LightColor;
    vec4 specular = spec * SpecularColor * LightColor;

    return clamp(ambient + diffuse + specular, 0.0, 1.0);
}

vec4 cartoonShading() {
    vec3 norm = normalize(v_Normal);
    vec3 lightDir = normalize(LightPos - v_WorldPos);
    float diff = max(dot(norm, lightDir), 0.0);

    float threshold = 0.5;
    diff = step(threshold, diff);

    vec4 ambient = AmbientColor;
    vec4 diffuse = diff * DiffuseColor * LightColor;

    return vec4(clamp(ambient + diffuse, 0.0, 1.0), 1.0);
}

vec4 goochShading() {
    vec3 norm = normalize(v_Normal);
    vec3 lightDir = normalize(LightPos - v_WorldPos);
    float diff = max(dot(norm, lightDir), 0.0);

    vec3 viewDir = normalize(WorldPosition - v_WorldPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), Shininess);

    vec4 cool = vec4(0.0, 0.0, 1.0, 1.0); // Blue
    vec4 warm = vec4(1.0, 1.0, 0.0, 1.0); // Yellow

    vec4 ambient = AmbientColor;
    vec4 diffuse = mix(cool, warm, diff) * DiffuseColor * LightColor;
    vec4 specular = spec * SpecularColor * LightColor;

    return vec4(clamp(ambient + diffuse + specular, 0.0, 1.0), 1.0);
}


void main() {
    if (mode == 0)
    FragColor = vec4(AmbientColor, 1.0);
    else if (mode == 1)
    FragColor = phongShading();
    else if (mode == 2)
    FragColor = cartoonShading();
    else if (mode == 3)
    FragColor = goochShading();
    else
    FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}