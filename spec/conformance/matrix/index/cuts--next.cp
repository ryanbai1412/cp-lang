// expect: ok, output "YES"
domain V = 0..3
print(yes(V.cuts(0).next! == V.cuts(1)))
