// expect: ok, output "0 1 3"
let e: List<Int> = []
print(e.deduped().len, [4, 4, 4].deduped().len, [1, 2, 1].deduped().len)
