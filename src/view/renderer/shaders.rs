pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;

            layout(location = 0) out vec2 tex_coords;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            vec2 tex_corners[4] = vec2[](
                vec2(0, 0),
                vec2(0, 1),
                vec2(1, 0),
                vec2(1, 1)
            );

            void main() {
                tex_coords = tex_corners[gl_VertexIndex % 4];
                vec4 position = vec4(position, 1);
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

            layout(location = 0) in vec2 tex_coords;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 1) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, tex_coords);
            }
        "
    }
}
