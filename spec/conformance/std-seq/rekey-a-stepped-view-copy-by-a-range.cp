// expect: ok, output "3 1\n3 1"
// rule: 04b.1.7
let os = [Some(3), None, Some(1), Some(4)]
let evens = List(os.even).rekey(1..3)
print(evens[1]!, evens[2]!)
print(List(evens))
