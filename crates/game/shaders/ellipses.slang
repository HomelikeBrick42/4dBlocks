import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Ellipse
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
StructuredBuffer<Ellipse> ellpises;

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

    let ellpise = ellpises[instance_id];

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    out.uv = float2(x, y);

    let position = float2(ellpise.x, ellpise.y);
    let size = float2(ellpise.width, ellpise.height);
    var point = position + (out.uv - 0.5) * size;
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
    let coord = in.uv * 2.0 - 1.0;
    if (dot(coord, coord) > 1.0)
        discard;

    var out : FragmentOutput;

    let ellpise = ellpises[in.instance_id];
    out.color = float4(ellpise.r, ellpise.g, ellpise.b, ellpise.a);

    return out;
}
