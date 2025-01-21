use wgpu_bootstrap::{
    cgmath, egui,
    util::{
        geometry::icosphere,
        orbit_camera::{CameraUniform, OrbitCamera},
    },
    wgpu::{self, util::DeviceExt},
    App, Context,
};



//Sommet avec Position 3D et Couleur 3D
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}



//Particules avec position 3D
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 3],
    color: [f32; 3], // Couleur de la particul
}


impl Instance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3, // Position
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3, // Couleur
                },
            ],
        }
    }
}





//Structure principale de l'App
pub struct InstanceApp {
    vertex_buffer: wgpu::Buffer,
    // instance_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    num_indices: u32,
    num_instances: u32,
    camera: OrbitCamera,
    compute_pipeline: wgpu::ComputePipeline, // Ajoutez ce champ
    compute_bind_group: wgpu::BindGroup,     // Ajoutez ce champ
    instance_storage_buffer: wgpu::Buffer,  // Ajoutez ce champ
}

impl InstanceApp {
    //Initialisation de l'application
    pub fn new(context: &Context) -> Self {
        let (positions, indices) = icosphere(2);

        let vertices: Vec<Vertex> = positions
            .iter()
            .map(|position| Vertex {
                position: (*position * 0.02).into(),
                color: [1.0, 0.0, 0.0],
            })
            .collect();

        let index_buffer = context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            });

        // let instances: Vec<Instance> = positions
        //     .iter()
        //     .map(|position| Instance {
        //         position: (*position).into(),
        //     })
        //     .collect();

        let num_indices = indices.len() as u32;
        // let num_instances = instances.len() as u32;

        let vertex_buffer =
            context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // let instance_buffer =
        //     context
        //         .device()
        //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //             label: Some("Instance Buffer"),
        //             contents: bytemuck::cast_slice(instances.as_slice()),
        //             usage: wgpu::BufferUsages::VERTEX,
        //         });

        let shader = context
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let camera_bind_group_layout = context
            .device()
            .create_bind_group_layout(&CameraUniform::desc());

        let pipeline_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc(), Instance::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.format(),
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: context.depth_stencil_format(),
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        let aspect = context.size().x / context.size().y;
        let mut camera = OrbitCamera::new(context, 45.0, aspect, 0.1, 100.0);
        camera
            .set_polar(cgmath::point3(3.0, 0.0, 0.0))
            .update(context);






 // Augmenter la taille de la sphère
let mut sphere_instances: Vec<Instance> = positions
.iter()
.map(|position| Instance {
    position: (*position * 1.0).into(), // Augmenter la taille
    color: [1.0, 0.0, 0.0],
})
.collect();

// Création d'un plan plus compact
let plane_size = 5; // Taille réduite
let mut plane_instances = Vec::new();
for x in -plane_size..plane_size {
    for z in -plane_size..plane_size {
        plane_instances.push(Instance {
            position: [x as f32 * 0.2, 1.3, z as f32 * 0.2], // Position encore plus proche de la sphère
            color: [0.0, 0.0, 1.0]
        });
    }
}

// Combinez les deux ensembles d'instances
sphere_instances.extend(plane_instances);
let num_instances = sphere_instances.len() as u32;







        
            

        //GPU
        let instance_storage_buffer = context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Storage Buffer"),
            contents: bytemuck::cast_slice(&sphere_instances),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let compute_shader = context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()), // Charge le fichier WGSL
        });
        





        
        

        let compute_bind_group_layout = context.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform, // Caméra comme buffer uniforme
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, // Buffer de stockage
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        println!("Camera buffer binding created: {:?}", camera.buffer_binding());
println!("Instance storage buffer binding created: {:?}", instance_storage_buffer.as_entire_binding());
        
let compute_bind_group = context.device().create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &compute_bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry {
            binding: 0, // Binding pour le buffer de la caméra
            resource: camera.buffer_binding(), // Fournit le buffer de la caméra
        },
        wgpu::BindGroupEntry {
            binding: 1, // Binding pour le buffer des positions d'instances
            resource: instance_storage_buffer.as_entire_binding(), // Fournit le buffer des instances
        },
    ],
    label: Some("Compute Bind Group"),
});


        let compute_pipeline_layout = context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = context.device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "cs_main",
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        
 



        Self {
            vertex_buffer,
            // instance_buffer,
            index_buffer,
            render_pipeline,
            num_indices,
            num_instances,
            camera,
            instance_storage_buffer, // Ajoutez ceci
            compute_pipeline, 
            compute_bind_group,
        }        
    }



    fn compute(&self, context: &Context) {
        let mut encoder = context.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Command Encoder"),
        });
    
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
    
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[]); // Utilisation correcte
        compute_pass.dispatch_workgroups((self.num_instances as f32 / 64.0).ceil() as u32, 1, 1);
    
        drop(compute_pass);
        context.queue().submit(Some(encoder.finish()));
    }

}




impl App for InstanceApp {
    fn input(&mut self, input: egui::InputState, context: &Context) {
        self.camera.input(input, context);
        // Met à jour les positions des particules
        self.compute(context);
    }

    fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
         
        render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_storage_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.num_instances);
    }
}