// expect: ok, output "1"
domain V = 0..3
print(V.cuts.count(x => x == V.cuts(1)))
