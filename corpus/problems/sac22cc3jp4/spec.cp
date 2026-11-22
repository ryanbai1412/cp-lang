// sac22cc3jp4: convert each number from one base to another.
for _ in 0..read<1..=1_000>():
    let (from, s, to) = read<(2..=36, Str, 2..=36)>()
    print(s.parse_int(base: from)!.to_str(base: to))
