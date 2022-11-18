pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 v_position;
            layout(location = 1) in float v_light;

            layout(location = 0) out vec2 f_tex_coords;
            layout(location = 1) out float f_light;

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
                // View transformations
                vec4 position = vec4(v_position, 1);
                mat4 worldview = uniforms.view * uniforms.world;

                // Fragment properties
                f_tex_coords = tex_corners[gl_VertexIndex % 4];
                f_light = v_light;

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

            layout(location = 0) in vec2 f_tex_coords;
            layout(location = 1) in float f_light;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 1) uniform sampler2D tex;

            const vec3 light_color = vec3(1.0, 1.0, 1.0);

            const vec3 ambient_color = vec3(1.0, 1.0, 1.0);
            const float ambient_strength = 0.2;

            void main() {
                vec3 ambient = ambient_color * ambient_strength;
                vec3 local = light_color * f_light;

                vec4 texture_color = texture(tex, f_tex_coords);
                vec4 texture = vec4(ambient + local, 1) * texture_color;

                f_color = texture;
            }
        "
    }
}
