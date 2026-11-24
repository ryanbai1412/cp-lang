// expect: ok, output "0 1 2\n3 4 5"
// rule: std.grid GridRows prints one row per line, like the grid
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
print(g.rows)
