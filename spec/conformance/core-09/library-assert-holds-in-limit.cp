// expect: ok, output "1048576"
// rule: 09.4.5
// std's always-on check in `Subset`'s `let count` passes inside its limit of 25 values; only a larger domain traps, at the user's line.
domain V = 0..20
let dp: [Subset<V>]Int = [Subset<V>] _ => 0
print(dp.len)
