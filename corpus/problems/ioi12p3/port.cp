// IOI 2012 Crayfish Scrivener: a text, empty at first. "T c" types the
// letter c at the end, "U x" undoes the last x commands (an undo is a
// command too, and can be undone), and "P i" prints the letter at position
// i, counting from 0. P is not a command. The input is the number of lines
// and then the lines.
//
// The original is a persistent segment tree with one root per command.
// Every text is some earlier text plus one letter, so the texts form a
// tree: a node is a text, its parent is the text without the last letter.
// A command only names a node, and an undo names the node of an earlier
// command. "P i" is the ancestor of the current node with i + 1 letters;
// each node keeps one long jump upwards, chosen so that the climb takes
// O(log n) steps and a new node needs no table.
let n: Int = read()
domain Text = 0..=n  // Text.first is the empty text
var len: [Text]Int = 0
var last: [Text]Char = ' '
var up = [Text] t => t
var jump = [Text] t => t

var texts = 0  // texts made so far, not counting the empty one
var history: List<Text> = [Text.first]
for _ in 0..n:
    let op: Char = read()
    if op == 'T':
        let letter: Char = read()
        let (old, now) = (history.last, Text.nth(texts + 1))
        texts += 1
        len[now] = len[old] + 1
        last[now] = letter
        up[now] = old
        let j = jump[old]
        jump[now] = if len[old] - len[j] == len[j] - len[jump[j]]: jump[j] else: old
        history.push(now)
    elif op == 'U':
        let x: Int = read()
        history.push(history[history.len - 1 - x])
    else:
        let i: Int = read()
        var v = history.last
        while len[v] != i + 1:
            v = if len[jump[v]] > i: jump[v] else: up[v]
        print(last[v])
