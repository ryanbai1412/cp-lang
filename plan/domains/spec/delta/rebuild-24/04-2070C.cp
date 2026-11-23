// CF 2070C Limited Repainting. With penalty at most p only cells with a > p must end
// right; the paint needed is the number of blue runs among those cells.
@!multitest
let (n, k): (1..=3e5, 0..=n) = read()
let s = read<Str>()
let a = read<[n]1..=1e9>()

fn ops_needed(p: Int) -> Int:
    var runs = 0
    var in_blue = false
    for i in 0..n if a[i] > p:
        if s[i] == 'B':
            if not in_blue: runs += 1
            in_blue = true
        else: in_blue = false
    runs

print((0..=1e9).first_true(p => ops_needed(p) <= k)!)
