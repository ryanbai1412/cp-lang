// expect: ok, output "12"
// rule: 04.2.11
// A table over a product may take one parameter, the cell tuple (rebuild round 16 R16-TABLE-TUPLE-PARAM).
domain R = 0..2
domain C = 0..3
let t = [R, C] p => p.0 * 10 + p.1
print(t[1, 2])
