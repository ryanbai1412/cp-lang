// expect: ok, output "YES"
enum D = N | E | S | W
let o: ?D = None
let s: ?D = D.E
print(yes(o == None and s! == D.E))
