// expect: E0302
// rule: 04b.1.7
let os = [Some(3), None, Some(1), Some(4)]
print(List(os.keyed(keys: List(os.even).rekey(os.even.keys)))) // <- here
