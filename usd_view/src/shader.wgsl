struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ModelUniform {
    model: mat4x4<f32>,
};

@group(1) @binding(1)
var<uniform> model: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) view_position: vec3<f32>,
    @location(1) view_normal: vec3<f32>,
};

fn inverse(m: mat3x3<f32>) -> mat3x3<f32> {
    let a = m[0][0];
    let b = m[0][1];
    let c = m[0][2];
    let d = m[1][0];
    let e = m[1][1];
    let f = m[1][2];
    let g = m[2][0];
    let h = m[2][1];
    let i = m[2][2];
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    let inv_det = 1.0 / det;
    return mat3x3<f32>(
        vec3<f32>(e * i - f * h, c * h - b * i, b * f - c * e),
        vec3<f32>(f * g - d * i, a * i - c * g, c * d - a * f),
        vec3<f32>(d * h - e * g, b * g - a * h, a * e - b * d),
    ) * inv_det;
}

fn model_to_view_normal(n: vec3<f32>) -> vec3<f32> {
    let m4x4 = camera.view * model.model;
    let m = mat3x3<f32>(m4x4[0].xyz, m4x4[1].xyz, m4x4[2].xyz);
    return normalize(transpose(inverse(m)) * n);
}

@vertex
fn vs_main(
    vin: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let pos = camera.view * model.model * vec4<f32>(vin.position, 1.0);
    out.clip_position = camera.projection * pos;
    out.view_position = pos.xyz / pos.w;
    out.view_normal = model_to_view_normal(vin.normal);
    return out;
}

@fragment
fn fs_main(fin: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_color = vec3<f32>(0.1, 0.1, 0.1);
    let k_diffuse = 0.7;
    let diffuse = max(dot(normalize(fin.view_normal), light_dir), 0.0);
    let color = k_diffuse * diffuse * light_color + ambient_color;
    return vec4<f32>(color, 1.0);
}
