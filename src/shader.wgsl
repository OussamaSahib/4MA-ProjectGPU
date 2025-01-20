struct CameraUniform {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;
// Storage Buffer pour les positions des instances
@group(0) @binding(1) var<storage, read_write> instance_positions: array<vec3<f32>>;
// Uniform Buffer pour les paramètres de simulation
@group(0) @binding(2) var<uniform> simulation_params: SimulationParams;

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
    out.color = instance.color;
    out.clip_position = camera.proj * camera.view * vec4<f32>(model.position + instance.pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0); // Appliquer la couleur
}






struct SimulationParams {
    sphere_radius: f32,
    spring_stiffness: f32,
    damping_factor: f32,
    _padding: f32, // Alignement
};

// Définir les constantes globales
const GRAVITY: vec3<f32> = vec3<f32>(0.0, -9.8, 0.0);
const TIME_STEP: f32 = 0.016;



//Ajoutez le Compute Shader dans shader.wgsl pour déplacer les particules sur le GPU.
@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;

    if (index < arrayLength(&instance_positions)) {
        var position = instance_positions[index];
        var velocity: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0); // Initialiser la vitesse

        // Appliquer la gravité
        velocity += GRAVITY * TIME_STEP;

        // Collision avec la sphère
        let sphere_center: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0); // Position de la sphère
        let to_center = position - sphere_center;
        let distance = length(to_center);

        if (distance < simulation_params.sphere_radius) {
            let normal = normalize(to_center);
            position = sphere_center + normal * simulation_params.sphere_radius;
            velocity = velocity - dot(velocity, normal) * normal; // Réduire la composante normale
        }

        // Mise à jour de la position
        position += velocity * TIME_STEP;

        // Empêcher de descendre en dessous du sol
        if (position.y < 0.0) {
            position.y = 0.0;
            velocity.y = 0.0;
        }

        instance_positions[index] = position;
    }
}