// expect: ok, output "3 2"
// rule: 07.11.3
let g = [1, 2, 3, 4, 5].group(by: x => x % 2)
print(g[1].len, g[0].len)
