// expect: ok, output "1\n2"
// rule: 04b.1.12 07.4.12
// A row `g[r]` keeps the column domain, so its keys are `C` positions, not 0-based offsets: on `C = 1..=3` the 'B' of row 1 is at `C(1)` (rebuild round 32 FL7, a silent wrong answer).
domain R = 0..2
domain C = 1..=3
let g: [R, C]Char = [R, C] (r, c) => if Int(r) + Int(c) == 2: 'B' else: 'W'
let row = g[R(1)]
print(row.keys.find(c => row[c] == 'B')!)
print([c for (c, x) in g[R(0)].items if x == 'B'])
