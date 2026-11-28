// expect: ok, output "5 -1"
// input: "5\n"
// engines: next
let x = read<?Int>()
let y = read<?Int>()
let a = if let v = x: v else: 0
let b = if let v = y: v else: -1
print(a, b)
