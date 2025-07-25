struct LightView {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> light: LightView;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_mat = mat4x4<f32> (
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );
    var out: VertexOutput;
    let world_pos = model_mat * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_position = light.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> FragmentOutput {
    let light_distance = distance(in.world_pos, light.position);
    var out: FragmentOutput;
    out.depth = light_distance / 200.0;
    return out;
}
