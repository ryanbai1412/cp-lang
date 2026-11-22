// expect: ok, output "1 -1 0"
let e: List<Int> = []
print([3, 9, 2, 9].argmax() ?? -1, e.argmax() ?? -1, [5, 5].argmin() ?? -1)
