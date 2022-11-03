pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec3 position;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                vec4 position = vec4(position.x, -position.y, position.z, 1);
                mat4 worldview = uniforms.view * uniforms.world;
                gl_Position = uniforms.proj * worldview * position;
            }
        ",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            layout(location = 0) out vec4 f_color;

            void main() {
                float intensity = 0.5;
                f_color = vec4(intensity, intensity, intensity, 1.0);
            }
        "
    }
}
