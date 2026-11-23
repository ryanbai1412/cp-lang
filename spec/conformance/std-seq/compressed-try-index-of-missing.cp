// expect: ok, output "YES YES"
let (domain C, xs) = compress([30, 10])
print(yes(xs.try_index_of(20) == None), yes(xs.try_index_of(30) != None))
