import include.camera;
import include.ray;

[vk::binding(0, 0)]
WTexture2D main_texture;

[vk::binding(1, 0)]
ConstantBuffer<Camera> camera;

[vk::binding(0, 1)]
StructuredBuffer<uint> chunk;

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

    var color = sky_color(ray);
    if (let hit = hit_scene(ray))
    {
        var sun_amount = dot(hit.normal, SUN_DIRECTION) * 0.5 + 0.5;

        var sun_ray : Ray;
        sun_ray.origin = hit.position + hit.normal * 0.001;
        sun_ray.direction = SUN_DIRECTION;
        if (let sun_hit = hit_scene(sun_ray))
            sun_amount = 0.0;

        color = hit.color * max(0.5, sun_amount);
    }
    main_texture.Store(global_index.xy, float4(color, 1.0));
}

static const float4 SUN_DIRECTION = normalize(float4(-0.3, 1.0, 0.4, 0.1));

float3 sky_color(Ray ray)
{
    if (dot(SUN_DIRECTION, ray.direction) > 0.99)
        return float3(1.0, 1.0, 1.0);

    let up_sky_color = float3(0.4, 0.5, 0.8);
    let down_sky_color = float3(0.1, 0.2, 0.4);
    return lerp(down_sky_color, up_sky_color, ray.direction.y * 0.5 + 0.5);
}

Optional<Hit> hit_scene(Ray ray)
{
    return hit_voxels(ray);
}

struct Hit
{
    float distance;
    float4 position;
    float4 normal;
    float3 color;
}

static const uint CHUNK_SIZE = 128;

uint get_block_index(int4 position)
{
    return uint(position.x + position.y * CHUNK_SIZE + position.z * CHUNK_SIZE * CHUNK_SIZE + position.w * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
}

Optional<Hit> hit_voxels(Ray ray)
{
    let ray_step_size_per_unit_axis = float4(
        length(ray.direction / ray.direction.x),
        length(ray.direction / ray.direction.y),
        length(ray.direction / ray.direction.z),
        length(ray.direction / ray.direction.w));
    var map_check = int4(floor(ray.origin));
    var step : int4;
    var ray_lengths_per_axis : float4;
    for (var i = 0u; i < 4u; i += 1u)
    {
        if (ray.direction[i] < 0.0)
        {
            step[i] = -1;
            ray_lengths_per_axis[i] = (ray.origin[i] - float(map_check[i])) * ray_step_size_per_unit_axis[i];
        }
        else
        {
            step[i] = 1;
            ray_lengths_per_axis[i] = (float(map_check[i] + 1) - ray.origin[i]) * ray_step_size_per_unit_axis[i];
        }
    }

    var distance = 0.0;
    while (distance < 64.0)
    {
        var smallest_length = 0u;
        for (var i = 0u; i < 4u; i += 1u)
            if (step[i] != 0 && ray_lengths_per_axis[i] < ray_lengths_per_axis[smallest_length])
                smallest_length = i;

        map_check[smallest_length] += step[smallest_length];
        distance = ray_lengths_per_axis[smallest_length];
        ray_lengths_per_axis[smallest_length] += ray_step_size_per_unit_axis[smallest_length];

        if (all(map_check >= int4(0)) && all(map_check < int4(CHUNK_SIZE)))
        {
            let index = get_block_index(map_check);
            let material = chunk[index];
            if (material != 0)
            {
                var hit : Hit;
                hit.distance = distance;
                hit.position = ray.origin + ray.direction * distance;
                hit.normal[smallest_length] = -step[smallest_length];
                hit.color = float3(1.0);
                return hit;
            }
        }
    }

    return none;
}
