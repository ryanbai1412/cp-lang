let n = read<1..=1e9>()
fn at_least(m: Int) -> Bool:
    print("?", m)
    read<Str>() == ">="
print("!", (1..=n).last_true(m => at_least(m))!)
