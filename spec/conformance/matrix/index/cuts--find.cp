// expect: ok, output "YES"
domain V = 0..3
print(yes(V.cuts.find(x => x == V.cuts(1))! == V.cuts(1)))
