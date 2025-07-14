@group(0) @binding(0) var current_state: texture_2d<f32>;
@group(0) @binding(1) var next_state: texture_storage_2d<r32float, write>;

const GRID_SIZE: u32 = 64u;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Bounds check
    if (x >= GRID_SIZE || y >= GRID_SIZE) {
        return;
    }
    
    let coord = vec2<i32>(i32(x), i32(y));
    
    // Count living neighbors
    var neighbors = 0.0;
    
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) {
                continue;
            }
            
            // Wrap around edges (toroidal topology)
            let nx = (i32(x) + dx + i32(GRID_SIZE)) % i32(GRID_SIZE);
            let ny = (i32(y) + dy + i32(GRID_SIZE)) % i32(GRID_SIZE);
            let neighbor_coord = vec2<i32>(nx, ny);
            
            neighbors += textureLoad(current_state, neighbor_coord, 0).r;
        }
    }
    
    let current_cell = textureLoad(current_state, coord, 0).r;
    
    // Conway's Game of Life rules:
    // 1. Any live cell with 2-3 neighbors survives
    // 2. Any dead cell with exactly 3 neighbors becomes alive
    // 3. All other cells die or stay dead
    
    var next_value = 0.0;
    if (current_cell > 0.5) {
        // Live cell
        if (neighbors >= 1.5 && neighbors <= 3.5) {
            next_value = 1.0;  // Survives
        }
    } else {
        // Dead cell
        if (neighbors >= 2.5 && neighbors <= 3.5) {
            next_value = 1.0;  // Birth
        }
    }
    
    textureStore(next_state, coord, vec4<f32>(next_value, 0.0, 0.0, 1.0));
}