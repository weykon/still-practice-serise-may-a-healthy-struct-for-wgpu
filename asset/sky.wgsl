struct Uniforms {
    width : u32,
    height : u32,
}
@group(0) @binding(0) var<uniform> uniforms : Uniforms;

alias TriangleVertices = array<vec2f, 6>;
var<private> vertices : TriangleVertices = TriangleVertices(
vec2f(-1.0, 1.0),
vec2f(-1.0, -1.0),
vec2f(1.0, 1.0),
vec2f(1.0, 1.0),
vec2f(-1.0, -1.0),
vec2f(1.0, -1.0),
);

struct Ray {
    origin : vec3f,
    direction : vec3f,
}

@vertex fn display_vs(@builtin(vertex_index) vid : u32) -> @builtin(position) vec4f {
    return vec4f(vertices[vid], 0.0, 1.0);
}

fn sky_color(ray : Ray) -> vec3f {
    let t = 0.5 * (normalize(ray.direction).y + 1.0);
    // 对于白色增长是，正k=1的反比下向偏蓝色的方向渐变
    return (1. - t) * vec3(1.) + t * vec3(0.3, 0.5, 1.0);
}
@fragment fn display_fs(@builtin(position) pos : vec4f) -> @location(0) vec4f {

    let aspect_ratio = f32(uniforms.width) / f32(uniforms.height);

    //-1 的作用仅仅是为了达到的是 1920 下的最后一个点是 1919 才对的位置
    var uv = pos.xy / vec2f(f32(uniforms.width - 1u), f32(uniforms.height - 1u));

    //uv 为比率下的宽高占比了调整坐标位置再翻转y
    uv = (2. * uv - vec2(1.)) * vec2(aspect_ratio, -1.);

    let origin = vec3(0.);
    let focus_distance = 1.;

    let direction = vec3(uv, -focus_distance);
    let ray = Ray(origin, direction);

    return vec4(sky_color(ray), 1.);
}
