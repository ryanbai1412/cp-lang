// expect: ok, output "YES"
domain D = 0..4
let o: ?D = None
let s: ?D = D(1)
print(yes(o == None and s! == D(1)))
