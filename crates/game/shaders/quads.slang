import include.camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Quad
{
    float2 position;
    float2 size;
    float4 color;
    float2 uv_offset;
    float2 uv_size;
}

[vk::binding(0, 1)]
StructuredBuffer<Quad> quads;

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

    let quad = quads[instance_id];

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    out.uv = float2(x, y);

    var point = quad.position + (out.uv - 0.5) * quad.size;
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

    let quad = quads[in.instance_id];
    out.color = quad.color * texture.Sample(textureSampler, quad.uv_offset + quad.uv_size * in.uv);

    return out;
}
