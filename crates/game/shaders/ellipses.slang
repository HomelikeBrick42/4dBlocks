import include.ui_camera;

[vk::binding(0, 0)]
ConstantBuffer<Camera> camera;

struct Ellipse
{
    float2 position;
    float2 size;
    float4 color;
    float2 uv_offset;
    float2 uv_size;
}

[vk::binding(0, 1)]
StructuredBuffer<Ellipse> ellipses;

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

    let ellipse = ellipses[instance_id];

    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);
    out.uv = float2(x, y);

    var point = ellipse.position + (out.uv - 0.5) * ellipse.size;
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
    let coord = in.uv * 2.0 - 1.0;
    if (dot(coord, coord) > 1.0)
        discard;

    var out : FragmentOutput;

    let ellipse = ellipses[in.instance_id];
    out.color = ellipse.color * texture.Sample(textureSampler, ellipse.uv_offset + ellipse.uv_size * in.uv);

    return out;
}
