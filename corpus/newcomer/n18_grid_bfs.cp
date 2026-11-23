let (h, w): (1..=1000, 1..=1000) = read()
let g: [h, w]Char = read()
let dx = [1, -1, 0, 0]  // expect: missing dx_dy_arrays
let dy = [0, 0, 1, -1]
var dist: [h, w]Int = -1  // expect: sentinel_table
var qx: List<Int> = []
var qy: List<Int> = []
var head = 0  // expect: missing manual-queue-head-index
qx.push(0)
qy.push(0)
dist[0, 0] = 0
while head < qx.len:
    let x = qx[head]
    let y = qy[head]
    head += 1
    for d in 0..4:
        let nx = x + dx[d]
        let ny = y + dy[d]
        if nx >= 0 and nx < h and ny >= 0 and ny < w:  // expect: grid_bounds_check
            if g[nx, ny] == '.' and dist[nx, ny] == -1:
                dist[nx, ny] = dist[x, y] + 1
                qx.push(nx)
                qy.push(ny)
print(dist[h - 1, w - 1])
