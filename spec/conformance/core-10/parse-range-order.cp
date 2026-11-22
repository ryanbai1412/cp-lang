// expect: ok, output "-1 3"
// rule: 10.2.3
let bad = "5 2".parse<Range<Int>>()
let good = "2 5".parse<Range<Int>>()!
print(if bad == None: -1 else: 0, good.len)
