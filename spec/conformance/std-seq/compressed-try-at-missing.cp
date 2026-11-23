// expect: ok, output "YES YES"
let (domain C, xs) = compress([30, 10])
print(yes(xs.try_at(20) == None), yes(xs.try_at(30) != None))
