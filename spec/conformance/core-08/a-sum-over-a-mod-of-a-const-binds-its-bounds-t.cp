// expect: ok, output "3 6"
// rule: 08.2.2
// `sum<S: Seq<T>, T, U>` over `Mod<MOD>`: the list's `Seq<T>` impl binds `T` even though `Mod<MOD>` names a const (G7.3).
const MOD = 7
type Z = Mod<MOD>
let zs: List<Z> = [Z(1), Z(2)]
print(sum(zs), sum(z * Z(2) for z in zs))
