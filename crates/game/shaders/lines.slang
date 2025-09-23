import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Line
{
    float2 a;
    float2 b;
    float3 color;
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

    let a_to_b = normalize(line.b - line.a);
    let a_to_b_perp = float2(-a_to_b.y, a_to_b.x);

    let end_point = lerp(line.a, line.b, x);
    var point = end_point - a_to_b_perp * ((y - 0.5) * line.width);
    point.x /= camera.aspect;
    out.clip_position = float4(point, 0.0, 1.0);

    return out;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;

    let line = lines[in.instance_id];
    out.color = float4(line.color, 1.0);

    return out;
}
