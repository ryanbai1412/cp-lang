// expect: ok, output "YES"
let (domain D, comp) = compress([30, 10, 20, 10])
let o: ?D = None
let s: ?D = D.last!
print(yes(o == None and s! == D.last!))
