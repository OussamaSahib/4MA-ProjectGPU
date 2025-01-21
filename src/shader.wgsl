struct CameraUniform {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
// Storage Buffer pour les positions des instances
@group(0) @binding(1) var<storage, read_write> instance_positions: array<vec3<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct InstanceInput {
    @location(2) pos: vec3<f32>,
    @location(3) color: vec3<f32>, // Couleur de l'instance
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>, // Couleur transmise au fragment shader
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.color; // Couleur de l'instance
    out.clip_position = camera.proj * camera.view * vec4<f32>(model.position + instance.pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0); // Appliquer la couleur
}

// Compute Shader pour déplacer les particules
@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x; // ID de l'instance à traiter
    if (index < arrayLength(&instance_positions)) {
        let position = instance_positions[index];

        // Appliquer la gravité
        if (position.y > 0.0) {
            instance_positions[index].y -= 0.01;
        }

        // Empêcher les particules de descendre sous le sol
        if (instance_positions[index].y < 0.0) {
            instance_positions[index].y = 0.0;
        }
    }
}
