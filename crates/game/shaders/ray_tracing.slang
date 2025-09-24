[vk::binding(0, 0)]
WTexture2D main_texture;

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

    let color = float3(uv * 0.5 + 0.5, 0.0);
    main_texture.Store(global_index.xy, float4(color, 1.0));
}
