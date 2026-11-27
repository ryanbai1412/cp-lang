// expect: ok, output "3 4 7\n12"
// rule: R01.6.1 R11.1.1
// A `const` is fixed by its initializer; a domain whose ends read only `const`s hoists with them,
// so it is counted above the line that declares it.
const N = 3
const M = N * 2 + 1
print(E.count, V.count, M)
domain E = [N]
domain V = 1..=N + 1
fn twice(x: Int) -> Int = x * 2
const T = twice(6)
print(T)
