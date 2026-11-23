// expect: ok, output "8 4"
// The inline `if` binds loosest, so `else:` takes the rest of the expression (round 24, H12).
let c = true
print(if c: 8 else: 4 >> 1, (if c: 8 else: 4) >> 1)
