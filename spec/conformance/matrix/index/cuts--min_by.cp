// expect: ok, output "YES"
domain V = 0..3
print(yes(V.cuts.min(by: x => 0)! == V.cuts(0)))
