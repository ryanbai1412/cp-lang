// expect: ok, output "463946602398166496 1841896406815527455 463946602398166496"
// rule: 10.2.3 10.2.4
// input: "148037899192074579360"
// A token far longer than an `Int` reads modulo a `P` whose `10 * r` would overflow an `Int`.
type Z = Mod<2305843009213693951>
let a: Z = read()
print(a, "-148037899192074579360".parse<Z>()!, "148037899192074579360".parse<Z>()!)
