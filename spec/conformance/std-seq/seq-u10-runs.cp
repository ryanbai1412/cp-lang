// expect: ok, output "1 1\n2 2 2\n1 3\n2 4\naa\nb\nccc\n0 2\n2 3\n3 6"
// Acceptance for seq/U10 (runs of equal values or keys), handbook/product/std/sequences-usecases.md.
for r in [1, 1, 2, 2, 2].runs():
    print(r)
for r in [1, 3, 2, 4].runs(by: x => x % 2):
    print(r)
for r in "aabccc".runs():
    print(r)
for r in "aabccc".runs():
    print(r.keys.l, r.keys.r)
