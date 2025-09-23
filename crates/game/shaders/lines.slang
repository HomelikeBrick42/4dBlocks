import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Line
{
    float ax;
    float ay;
    float bx;
    float by;
    float r;
    float g;
    float b;
    float a;
    float width;
}

[vk::binding(0, 1)]
StructuredBuffer<Line> lines;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    uint instance_id;
    float2 uv;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_id: SV_InstanceID)
{
    var out : VertexOutput;
    out.instance_id = instance_id;

    let line = lines[instance_id];

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    out.uv = float2(x, y);

    let a = float2(line.ax, line.ay);
    let b = float2(line.bx, line.by);
    let a_to_b = normalize(b - a);
    let a_to_b_perp = float2(-a_to_b.y, a_to_b.x);

    let end_point = lerp(a, b, x);
    var point = end_point - a_to_b_perp * ((y - 0.5) * line.width);
    point.x /= camera.aspect;
    out.clip_position = float4(point, 0.0, 1.0);

    return out;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[vk::binding(0, 2)]
Texture2D texture;
[vk::binding(1, 2)]
SamplerState textureSampler;

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;

    let line = lines[in.instance_id];
    out.color = float4(line.r, line.g, line.b, line.a) * texture.Sample(textureSampler, in.uv);

    return out;
}
