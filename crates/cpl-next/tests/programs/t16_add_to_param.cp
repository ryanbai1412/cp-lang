// expect: ok, output "6"
// Open design question: a list of functions received as a parameter gets a new function pushed.
fn add(fs: List<Int -> Int>, k: Int) -> List<Int -> Int>:
    var out = fs
    out.push(x => x + k)
    out
var start: List<Int -> Int> = []
let fs = add(start, 5)
print(fs[0](1))
