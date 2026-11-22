// expect: ok, output "5"
// rule: 02.4.6 04.4.27
domain V = 1..=4
var dist: [V]?Int = None
dist[V.first!] = 3
dist[V.last!] min= dist[V.first!] + 2
print(dist[V.last!] ?? -1)
