// NOI 2003 Editor: a text, empty at first, and a cursor between two of its
// characters. "Move k" puts the cursor after the k-th character, "Prev" and
// "Next" step it, "Insert n" inserts the n characters that follow at the
// cursor, "Delete n" removes the n characters after it and "Get n" prints
// them. The cursor stays where it is through an edit.
//
// The original uses a rope. There are at most 4000 edits, so a piece table
// is enough: the text is a list of strings, cut wherever an edit has landed,
// and an edit replaces a few whole pieces. Moving the cursor costs nothing.
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
    let s = pieces[i]
    pieces = pieces[0..i] + [s[0..p - before], s[p - before..s.len]] + pieces[i + 1..pieces.len]
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
                if a < b: shown += s[a - at..b - at]
                at += s.len
            print(shown)
