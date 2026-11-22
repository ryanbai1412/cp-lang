// expect: ok, output "YES"
domain V = 0..3
let o: ?V.cuts = None
let s: ?V.cuts = V.cuts(1)
print(yes(o == None and s! == V.cuts(1)))
