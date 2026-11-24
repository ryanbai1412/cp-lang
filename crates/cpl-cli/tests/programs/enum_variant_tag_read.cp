// A payload-free enum reads by each variant's `@tag` (R02.7.2), as a word
// or a character, alone, in a tuple and in a table; it prints by its tag.
enum Run:
    @tag("first") First
    @tag("second") Second
enum Cell = @tag('.') Empty | @tag('#') Wall
let r: Run = read()
let (k, s): (Int, Run) = read()
let n: 1..=3 = read()
domain Col = 1..=n
let row: [Col]Cell = read()
print(yes(r == First), k, s)
print(count(c == Wall for c in row))
print(row)
