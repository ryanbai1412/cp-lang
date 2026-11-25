// expect: E0603
// rule: R05.7.8
// A lent closure stored in a list that is returned.
fn make() -> List<() -> Int>:
    var c = 0
    var fs: List<() -> Int> = List.new()
    let f = () => { c += 1; c }
    fs.push(f)
    fs  // <- here
let fs = make()
print(fs[0]())
