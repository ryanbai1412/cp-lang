// expect: ok, output "2 3 YES"
// rule: 02.4.1
// `Some(3)` where a `?Mod<7>` is wanted is `Some(Mod<7>(3))`, as `let z: Mod<7> = 3` is `Mod<7>(3)`.
let y: ?Mod<7> = Some(3)
let x = Some(Mod<7>(9))
print(x!.v, y!.v, yes(x == Some(2)))
