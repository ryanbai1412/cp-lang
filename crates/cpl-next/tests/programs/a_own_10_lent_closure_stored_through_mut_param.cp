// expect: E0603
// rule: R05.10.2, R8
// Storing a lent closure into a list that outlives the var (through a mut parameter).
fn stash(mut fs: List<() -> Int>, f: () -> Int):
    fs.push(f)
fn fill(mut fs: List<() -> Int>):
    var c = 10
    stash(mut fs, () => { c += 1; c })  // <- here
var fs: List<() -> Int> = List.new()
fill(mut fs)
print(fs[0]())
