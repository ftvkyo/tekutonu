pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;

            layout(location = 0) out vec2 tex_coords;
            layout(location = 1) out vec3 f_position;
            layout(location = 2) out vec3 f_normal;
            layout(location = 3) out vec3 light_position;

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

            const vec4 lightPos = vec4(5);

            void main() {
                // Texture coordinates
                tex_coords = tex_corners[gl_VertexIndex % 4];

                // View transformations
                vec4 position = vec4(position, 1);
                mat4 worldview = uniforms.view * uniforms.world;

                light_position = (uniforms.world * lightPos).xyz;

                // Fragment properties
                f_position = (uniforms.world * position).xyz;
                f_normal = transpose(inverse(mat3(uniforms.world))) * normal;

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
            layout(location = 1) in vec3 f_position;
            layout(location = 2) in vec3 f_normal;
            layout(location = 3) in vec3 light_position;

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 1) uniform sampler2D tex;

            const vec3 light_color = vec3(1.0, 1.0, 1.0);

            const vec3 ambient_color = vec3(1.0, 1.0, 1.0);
            const float ambient_strength = 0.2;

            void main() {
                vec3 ambient = ambient_color * ambient_strength;

                vec3 light_direction = normalize(light_position - f_position);
                float diff = max(0, dot(normalize(f_normal), light_direction));
                vec3 diffuse = vec3(diff * light_color);

                vec4 texture_color = texture(tex, tex_coords);
                vec4 texture = vec4(ambient + diffuse, 1) * texture_color;

                f_color = texture;
            }
        "
    }
}
