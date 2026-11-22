// expect: ok, output "1 2"
let (domain D, comp) = compress([30, 10, 20, 10])
print(Int(D(1).l), Int(D(1).r))
