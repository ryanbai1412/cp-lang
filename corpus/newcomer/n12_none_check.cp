let n: 1..=100 = read()
let a: [0..n]Int = read()
var first: ?Int = None
for x in a:  // expect: search_loop
    if x % 2 == 0 and first == None:
        first = x
if first != None and first! > 10:  // expect: none_check_unwrap
    print(first!)
else:
    print(-1)
