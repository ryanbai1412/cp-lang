// expect: ok, output "YES"
let (domain D, comp) = compress([30, 10, 20, 10])
print(yes(D.find(x => x == D.last!)! == D.last!))
