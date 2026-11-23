// expect: ok, output "2 6 YES YES YES 6\n0 0 YES 1\n55 4"
// Acceptance for seq/U18 (count, sum, any, all, contains on any shape), handbook/product/std/sequences-usecases.md.
let a = [1, 2, 3]
let e: List<Int> = []
print(a.count(x => x > 1), a.sum(), yes(a.any(x => x > 2)), yes(a.all(x => x > 0)), yes(a.contains(2)), a.prod())
print(e.sum(), e.count(), yes(e.all(x => x > 0)), e.prod())
print((1..=10).sum(), (0..10).filter(x => x % 3 == 0).count())
