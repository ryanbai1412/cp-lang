// Source: NOI 2003 P2 (Text Editor), https://dmoj.ca/problem/noi03p2
// Problem: a text, empty at first, and a cursor. t commands: "Move k", "Prev", "Next" place the
//   cursor; "Insert n" inserts the n characters (ASCII 32-126) that follow, line breaks
//   ignored; "Delete n" removes and "Get n" prints the n characters after the cursor. At most
//   4000 inserts and deletes, 2 MB inserted, 3 MB printed.
// Solution: with so few edits a piece table is enough: the text is a list of strings, cut
//   wherever an edit lands, and an edit replaces a few whole pieces; moving is free. Each edit
//   and each Get costs O(pieces), a Get also O(n) to print; the original uses a rope.
let t: Int = read()
var pieces: List<Str> = []
var cursor = 0

// Make position p a piece boundary; the number of pieces before it.
fn cut(p: Int) -> Int:
    var (i, before) = (0, 0)
    while i < pieces.len and before + pieces[i].len <= p:
        before += pieces[i].len
        i += 1
    if before == p: return i
    let s = pieces[i].to_str()
    pieces = pieces[0..i].to_list() + [s[0..p - before].to_str(), s[p - before..s.len].to_str()] + pieces[i + 1..pieces.len].to_list()
    i + 1

for _ in 0..t:
    let cmd: Str = read()
    if cmd == "Prev":
        cursor -= 1
    elif cmd == "Next":
        cursor += 1
    else:
        let n: Int = read()
        if cmd == "Move":
            cursor = n
        elif cmd == "Insert":
            var text = ""
            while text.len < n:
                let word: Str = read()
                text += word
            let i = cut(cursor)
            pieces = pieces[0..i] + [text] + pieces[i..pieces.len]
        elif cmd == "Delete":
            let (i, j) = (cut(cursor), cut(cursor + n))
            pieces = pieces[0..i] + pieces[j..pieces.len]
        else:
            var (shown, at) = ("", 0)
            for s in pieces:
                let (a, b) = (max(cursor, at), min(cursor + n, at + s.len))
                if a < b: shown += s[a - at..b - at].to_str()
                at += s.len
            print(shown)
