// expect: ok, output "9"
// rule: R06 scopes
// Shadowing: an inner `let x` in a block does not change the outer x; a for var shadows and restores.
let x = 9
if x > 0:
    let x = 1
    for x in 0..3:
        let y = x
print(x)
