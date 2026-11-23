// expect: ok, output "1\n3"
// rule: 02.5.5
// `(u:, v:)` binds fields by name (rebuild round 6 H-RECORD-PATTERN).
domain V = 0..3
let es: [0..2](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]
for (u:, v:) in es:
    print(Int(u) + Int(v))
