// expect: ok, output "YES"
domain V = 0..3
print(yes(V.cuts(1).prev! == V.cuts(0) and V.cuts(0).prev == None))
