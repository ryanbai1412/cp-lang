// expect: ok, output "5 7\n2 4"
// rule: 07.11.3
// Each list of `group(by:)` keeps walk order (round 26, H11).
let g = [5, 2, 7, 4].group(by: x => x % 2)
print(g[1])
print(g[0])
