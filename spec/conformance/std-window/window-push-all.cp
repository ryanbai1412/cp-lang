// expect: ok, output "YES\n5 2"
var w = Window<Int, (Int, Int) -> Int>.new((a: Int, b: Int) => a + b)
print(yes(w.all == None))
w.push(2)
w.push(3)
print(w.all!, w.len)
