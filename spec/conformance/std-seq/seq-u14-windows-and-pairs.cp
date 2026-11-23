// expect: ok, output "3\n6\n9\n3\n7\n5\n1\n2"
// Acceptance for seq/U14 (fixed windows, chunks and adjacent pairs), handbook/product/std/sequences-usecases.md.
for v in [1, 2, 4, 5].windows(2):
    print(v.sum())
for c in [1, 2, 3, 4, 5].chunks(2):
    print(c.sum())
for (x, y) in [1, 2, 4].adjacent():
    print(y - x)
