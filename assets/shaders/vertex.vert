#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec3 v_WorldPos;
layout(location = 1) out vec3 v_Normal;
layout(location = 2) out vec2 v_Uv;
layout(location = 3) out vec3 cam_WorldPos;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
    mat4 View;
    mat4 InverseView;
    mat4 Projection;
    vec3 WorldPosition;
    float width;
    float height;
};

struct Mesh {
    mat3x4 Model;
    mat4 InverseTransposeModel;
    uint flags;
};


mat4 affine_to_square(mat3x4 affine) {
    return transpose(mat4(
        affine[0],
        affine[1],
        affine[2],
        vec4(0.0, 0.0, 0.0, 1.0)
    ));
}

#ifdef PER_OBJECT_BUFFER_BATCH_SIZE
layout(set = 1, binding = 0) uniform Mesh Meshes[#{PER_OBJECT_BUFFER_BATCH_SIZE}];
#else
layout(set = 1, binding = 0) readonly buffer _Meshes {
    Mesh Meshes[];
};
#endif // PER_OBJECT_BUFFER_BATCH_SIZE

void main() {
    vec4 worldPos = affine_to_square(Meshes[gl_InstanceIndex].Model) * vec4(Vertex_Position, 1.0);
    v_WorldPos = worldPos.xyz;
    v_Normal = mat3(Meshes[gl_InstanceIndex].InverseTransposeModel) * Vertex_Normal;
    v_Uv = Vertex_Uv;
    cam_WorldPos = WorldPosition;
    gl_Position = ViewProj * worldPos;
}