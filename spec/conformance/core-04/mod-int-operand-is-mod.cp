// expect: ok, output "0 3"
// rule: 04.4.28
type Z = Mod<7>
let before: [3]Z = [Z(5), Z(6), Z(1)]
print(before[1] + 1, 2 * before[0])
