import include.camera;
import include.ray;

[vk::binding(0, 0)]
WTexture2D main_texture;

[vk::binding(1, 0)]
ConstantBuffer<Camera> camera;

[shader("compute")]
[numthreads(16, 16, 1)]
void trace_rays(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    main_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    let uv = ((float2(global_index.xy) + 0.5) / float2(width, height)) * 2.0 - 1.0;

    var ray : Ray;
    ray.origin = camera.position;
    ray.direction = normalize(camera.right * uv.x * camera.aspect + camera.up * uv.y + camera.forward);

    let color = sky_color(ray);
    main_texture.Store(global_index.xy, float4(color, 1.0));
}

static const float4 SUN_DIRECTION = normalize(float4(-0.3, 1.0, 0.3, 0.0));

float3 sky_color(Ray ray)
{
    if (dot(SUN_DIRECTION, ray.direction) > 0.99)
        return float3(1.0, 1.0, 1.0);

    let up_sky_color = float3(0.4, 0.5, 0.8);
    let down_sky_color = float3(0.1, 0.2, 0.4);
    return lerp(down_sky_color, up_sky_color, ray.direction.y * 0.5 + 0.5);
}
