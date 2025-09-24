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

    let color = float3(ray.direction.xyz * 0.5 + 0.5);
    main_texture.Store(global_index.xy, float4(color, 1.0));
}
