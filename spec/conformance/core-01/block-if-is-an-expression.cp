// expect: ok, output "7"
// rule: 01.7.1
// A block `if`/`else` is an expression whose arms end in their value (rebuild round 18 H09).
let x = 3
let t = if x > 1:
    let y = x * 2
    y + 1
else:
    0
print(t)
