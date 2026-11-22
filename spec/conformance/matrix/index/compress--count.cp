// expect: ok, output "1"
let (domain D, comp) = compress([30, 10, 20, 10])
print(D.count(x => x == D.last!))
