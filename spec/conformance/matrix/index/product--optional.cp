// expect: ok, output "YES"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
let o: ?D = None
let s: ?D = D((1, 2))
print(yes(o == None and s! == D((1, 2))))
