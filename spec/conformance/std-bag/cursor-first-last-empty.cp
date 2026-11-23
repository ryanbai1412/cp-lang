// expect: ok, output "YES YES"
let e: List<Int> = []
let b = SortedBag.of(e)
print(yes(b.cursor_first() == None), yes(b.cursor_last() == None))
