// expect: E0503 at 2
// rule: 01.8.5
let g = [2, 2] (r, c) => r * 2 + c + 1
print(g[0][1])
