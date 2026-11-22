// expect: ok, output "3"
domain V = 0..3
print(V.cuts.filter(x => x != V.cuts(1)).count())
