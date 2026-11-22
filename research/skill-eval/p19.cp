let n = read<1..=1e9>()

fn ask(y: Int) -> Bool:
    print("?", y)
    read<Str>() == ">="

let ans = (1..=n).last_true(y => ask(y))!
print("!", ans)
