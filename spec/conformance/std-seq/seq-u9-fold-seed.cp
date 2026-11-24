// expect: ok, output "16 3"
// Acceptance for seq/U9 (a fold with a seed and a custom step), handbook/product/std/sequences-usecases.md: the seed is the named `init:`.
print([1, 2, 3].fold_from(10, (a: Int, x: Int) => a + x), "abca".fold_from(1, (n: Int, c: Char) => if c == 'a': n + 1 else: n))
