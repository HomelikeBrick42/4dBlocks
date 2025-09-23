import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Quad
{
    float x;
    float y;
    float width;
    float height;
    float r;
    float g;
    float b;
    float a;
}

[vk::binding(0, 1)]
StructuredBuffer<Quad> quads;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    uint instance_id;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_id: SV_InstanceID)
{
    var out : VertexOutput;
    out.instance_id = instance_id;

    let quad = quads[instance_id];

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    let uv = float2(x, y);

    let position = float2(quad.x, quad.y);
    let size = float2(quad.width, quad.height);
    var point = position + (uv - 0.5) * size;
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

    let quad = quads[in.instance_id];
    out.color = float4(quad.r, quad.g, quad.b, quad.a);

    return out;
}
