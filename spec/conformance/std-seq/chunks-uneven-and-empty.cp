// expect: ok, output "2 2 1\n0"
print([1, 2, 3, 4, 5].chunks(2).map(c => c.len).collect())
let e: List<Int> = []
print(e.chunks(3).count())
