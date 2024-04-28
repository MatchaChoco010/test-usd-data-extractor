struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct DirectionalLight {
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    angle: f32,
};
struct DirectionalLightsUniform {
    lights: array<DirectionalLight, 4>,
    count: u32,
};

@group(1) @binding(0)
var<uniform> directional_lights: DirectionalLightsUniform;

struct PointLight {
    position: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
};
struct PointLightsUniform {
    lights: array<PointLight, 16>,
    count: u32,
};

@group(1) @binding(1)
var<uniform> point_lights: PointLightsUniform;

struct SpotLight {
    position: vec3<f32>,
    intensity: f32,
    direction: vec3<f32>,
    angle: f32,
    color: vec3<f32>,
    softness: f32,
};
struct SpotLightsUniform {
    lights: array<SpotLight, 16>,
    count: u32,
};

@group(1) @binding(2)
var<uniform> spot_lights: SpotLightsUniform;

struct ModelUniform {
    model: mat4x4<f32>,
};

@group(2) @binding(0)
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

fn world_to_view_position(p: vec3<f32>) -> vec3<f32> {
    return (camera.view * vec4<f32>(p, 1.0)).xyz;
}

fn world_to_view_vector(v: vec3<f32>) -> vec3<f32> {
    return (camera.view * vec4<f32>(v, 0.0)).xyz;
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
    let mesh_color = vec3<f32>(0.5);
    let exposure = 0.25;

    var color = vec3<f32>(0.0);

    for (var i = 0; i < i32(directional_lights.count); i++) {
        let light = directional_lights.lights[i];
        let direction = normalize(world_to_view_vector(light.direction));
        let intensity = light.intensity * max(dot(fin.view_normal, direction), 0.0);
        color += light.color * intensity * mesh_color;
    }

    for (var i = 0; i < i32(point_lights.count); i++) {
        let light = point_lights.lights[i];

        let light_view_position = world_to_view_position(light.position);

        let direction = normalize(light_view_position - fin.view_position);

        let distance = length(light_view_position - fin.view_position);

        let epsilon = 0.001;
        let attenuation = 1.0 / (distance * distance + epsilon);

        let intensity = light.intensity * max(dot(fin.view_normal, direction), 0.0) * attenuation;
        color += light.color * intensity * mesh_color;
    }

    for (var i = 0; i < i32(spot_lights.count); i++) {
        let light = spot_lights.lights[i];

        let light_view_position = world_to_view_position(light.position);

        let direction = normalize(light_view_position - fin.view_position);

        let distance = length(light_view_position - fin.view_position);

        let epsilon = 0.001;
        let attenuation = 1.0 / (distance * distance + epsilon);

        let spot_direction = normalize(world_to_view_vector(light.direction));

        let outer_theta = light.angle;
        let inner_theta = light.angle * (1.0 - light.softness);
        let cos_s = dot(spot_direction, direction);
        let cos_u = cos(outer_theta);
        let cos_p = cos(inner_theta);
        let t = (cos_s - cos_u) / (cos_p - cos_u);
        let tt = clamp(t, 0.0, 1.0);
        let spot_factor = tt * tt;

        let intensity = light.intensity * max(dot(fin.view_normal, direction), 0.0) * attenuation * spot_factor;
        color += light.color * intensity * mesh_color;
    }

    return vec4<f32>(color * exposure, 1.0);
}
