// expect: ok, output "16 3"
// Acceptance for seq/U9 (a fold with a seed and a custom step), handbook/product/std/sequences-usecases.md: the seed is the named `init:`.
print([1, 2, 3].fold((a: Int, x: Int) => a + x, init: 10), "abca".fold((n: Int, c: Char) => if c == 'a': n + 1 else: n, init: 1))
