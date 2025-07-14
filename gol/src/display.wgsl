// Vertex shader for fullscreen quad
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Create fullscreen triangle (covers whole screen)
    var uv = vec2<f32>(
        f32((vertex_index << 1u) & 2u), 
        f32(vertex_index & 2u)
    );
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

// Conway state texture
@group(0) @binding(0) var conway_state: texture_2d<f32>;

const GRID_SIZE: u32 = 64u;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to grid coordinates
    let grid_pos = vec2<u32>(in.uv * f32(GRID_SIZE));
    
    // Bounds check
    if (grid_pos.x >= GRID_SIZE || grid_pos.y >= GRID_SIZE) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // Black for out of bounds
    }
    
    // Sample Conway state from texture
    let coord = vec2<i32>(i32(grid_pos.x), i32(grid_pos.y));
    let cell = textureLoad(conway_state, coord, 0).r;
    
    // Live cells = white, dead cells = black
    if (cell > 0.5) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);  // White = alive
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // Black = dead
    }
}