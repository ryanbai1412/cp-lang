let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], c))

fn best(from: Note.cuts, ends: Range<Note.cuts>) -> (Int, Note.cuts):
    let (p, at) = peak.fold(ends)
    (p - pre[from], at)

print(split_best(Note.cuts, range: s => s.shift(len), best: best).take(k).sum())
