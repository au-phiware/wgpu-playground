@group(0) @binding(0) var<storage, read> current_state: array<u32>;
@group(0) @binding(1) var<storage, read_write> next_state: array<u32>;

const GRID_SIZE: u32 = 64u;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Bounds check
    if (x >= GRID_SIZE || y >= GRID_SIZE) {
        return;
    }
    
    let index = y * GRID_SIZE + x;
    
    // Count living neighbors
    var neighbors = 0u;
    
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) {
                continue;
            }
            
            // Wrap around edges (toroidal topology)
            let nx = (x + GRID_SIZE + u32(dx)) % GRID_SIZE;
            let ny = (y + GRID_SIZE + u32(dy)) % GRID_SIZE;
            let neighbor_index = ny * GRID_SIZE + nx;
            
            neighbors += current_state[neighbor_index];
        }
    }
    
    let current_cell = current_state[index];
    
    // Conway's Game of Life rules:
    // 1. Any live cell with 2-3 neighbors survives
    // 2. Any dead cell with exactly 3 neighbors becomes alive
    // 3. All other cells die or stay dead
    
    if (current_cell == 1u) {
        // Live cell
        if (neighbors == 2u || neighbors == 3u) {
            next_state[index] = 1u;  // Survives
        } else {
            next_state[index] = 0u;  // Dies
        }
    } else {
        // Dead cell
        if (neighbors == 3u) {
            next_state[index] = 1u;  // Birth
        } else {
            next_state[index] = 0u;  // Stays dead
        }
    }
}