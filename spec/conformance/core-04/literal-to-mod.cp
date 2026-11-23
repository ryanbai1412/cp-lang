// expect: ok, output "3 0"
// rule: 04.4.28
type Z = Mod<7>
var ways: [0..=3]Z = 0
ways[0] = 10
print(ways[0], ways[1])
