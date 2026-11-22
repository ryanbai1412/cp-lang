// expect: ok, output "2"
let (domain D, comp) = compress([30, 10, 20, 10])
print(D.filter(x => x != D.last!).count())
