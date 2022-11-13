pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;

            layout(location = 0) out vec2 tex_coords;
            layout(location = 1) out vec3 v_normal;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            const vec2 tex_corners[4] = vec2[](
                vec2(0, 0),
                vec2(1, 0),
                vec2(1, 1),
                vec2(0, 1)
            );

            void main() {
                tex_coords = tex_corners[gl_VertexIndex % 4];
                mat4 worldview = uniforms.view * uniforms.world;
                v_normal = transpose(inverse(mat3(worldview))) * normal;
                gl_Position = uniforms.proj * worldview * vec4(position, 1);
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
            layout(location = 1) in vec3 v_normal;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 1) uniform sampler2D tex;

            const vec3 LIGHT = vec3(0.0, 0.0, 1.0);

            void main() {
                float brightness = dot(normalize(v_normal), normalize(LIGHT));
                vec4 dark_color = vec4(0.1, 0.1, 0.1, 1.0);
                vec4 regular_color = texture(tex, tex_coords);

                f_color = mix(dark_color, regular_color, brightness);
            }
        "
    }
}
