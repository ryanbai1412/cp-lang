// expect: ok, output "YES"
domain V = 0..3
print(yes(V.cuts.first! == V.cuts(0)))
