let (a, b): (0..=1000000006, 0..=1000000006) = read()
let d = (a - b + 1000000007) % 1000000007  // expect: neg_mod_fixup, repeated_literal
let p = a * b % 1000000007  // expect: missing int-mod-instead-of-Mod
print(d, p)
