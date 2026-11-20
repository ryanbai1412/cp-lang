import re

class Tok:
    __slots__ = ("kind", "val", "line", "col")
    def __init__(self, kind, val, line, col):
        self.kind, self.val, self.line, self.col = kind, val, line, col
    def __repr__(self):
        return f"{self.kind}:{self.val!r}@{self.line}"

class LexError(Exception):
    pass

OPS = ["..=", "..?", "**=", "...", "..", "=>", "->", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=",
       "%=", "**", "??", "<<", ">>", "&&", "||",
       "+", "-", "*", "/", "%", "<", ">", "=", "(", ")", "[", "]", "{", "}", ",", ":", ".",
       "?", "!", "|", "&", "^", "@", ";", "~"]
KEYWORDS = {"domain", "const", "type", "fn", "pure", "let", "var", "if", "elif", "else", "for", "in",
            "while", "return", "break", "continue", "assert", "and", "or", "not", "none",
            "true", "false", "match", "struct", "enum", "yield", "defer",
            "where", "ref", "inout"}
CONT_START = {"and", "or", "."}
NUM_RE = re.compile(r"\d[\d_]*(\.\d+)?([eE][+-]?\d+)?")
NAME_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")

def ascii_only(text, what, ln):
    """Char is one byte and Str is [S]Char: no UTF-8 anywhere (decision 1.6)."""
    for ch in text:
        if ord(ch) > 127:
            raise LexError(f"line {ln}: `{ch}` is not ASCII; a {what} literal holds single-byte ASCII characters only "
                           f"(Char is one byte, Str is a flat array of Char, there is no UTF-8)")

def lex(src):
    toks = []
    indents = [0]
    depth = 0
    lines = src.split("\n")
    for ln, raw in enumerate(lines, 1):
        line = raw.rstrip()
        i = 0
        stripped = line.lstrip()
        if not stripped or stripped.startswith("//"):
            continue
        ind = len(line) - len(stripped)
        if depth == 0:
            first = NAME_RE.match(stripped)
            word = first.group(0) if first else stripped[:1]
            is_cont = toks and toks[-1].kind == "NEWLINE" and ind > indents[-1] and (
                word in CONT_START and not stripped.startswith(".."))
            if is_cont:
                toks.pop()
            else:
                if ind > indents[-1]:
                    if not toks or toks[-1].kind != "NEWLINE" or toks[-2].val != ":":
                        # continuation of an expression line (hanging indent)
                        pass
                    indents.append(ind)
                    toks.append(Tok("INDENT", ind, ln, 0))
                while ind < indents[-1]:
                    indents.pop()
                    toks.append(Tok("DEDENT", ind, ln, 0))
                if ind != indents[-1]:
                    raise LexError(f"line {ln}: inconsistent indentation")
        i = ind
        n = len(line)
        while i < n:
            c = line[i]
            if c in " \t":
                i += 1
                continue
            if line.startswith("//", i):
                break
            col = i + 1
            m = NUM_RE.match(line, i)
            if m and not (c == "." ):
                txt = m.group(0)
                # don't eat '..' after an integer
                if m.group(1) and line.startswith("..", m.start(1)):
                    txt = line[i:m.start(1)]
                t = txt.replace("_", "")
                if "." in t or "e" in t.lower():
                    f = float(t)
                    val = int(f) if f == int(f) and abs(f) < 1e19 and "." not in t else f
                    if "e" in t.lower() and "." in t and f == int(f):
                        val = int(f)
                else:
                    val = int(t)
                toks.append(Tok("NUM", val, ln, col))
                i += len(txt)
                continue
            m = NAME_RE.match(line, i)
            if m:
                w = m.group(0)
                toks.append(Tok("KW" if w in KEYWORDS else "NAME", w, ln, col))
                i = m.end()
                continue
            if c == '"':
                j = i + 1
                buf = ""
                while j < n and line[j] != '"':
                    if line[j] == "\\":
                        buf += line[j:j + 2]
                        j += 2
                    else:
                        buf += line[j]
                        j += 1
                ascii_only(buf, "string", ln)
                toks.append(Tok("STR", buf, ln, col))
                i = j + 1
                continue
            if c == "'":
                j = line.index("'", i + 1 if line[i + 1] != "\\" else i + 3)
                body = line[i + 1:j]
                ch = {"\\n": "\n", "\\t": "\t", "\\'": "'", "\\\\": "\\"}.get(body, body)
                ascii_only(ch, "Char", ln)
                if len(ch) != 1:
                    raise LexError(f"line {ln}: a Char literal is exactly one byte, got '{body}'")
                toks.append(Tok("CHAR", ch, ln, col))
                i = j + 1
                continue
            for op in OPS:
                if line.startswith(op, i):
                    toks.append(Tok("OP", op, ln, col))
                    i += len(op)
                    if op in "([{":
                        depth += 1
                    elif op in ")]}":
                        depth -= 1
                    break
            else:
                raise LexError(f"line {ln}: unexpected {c!r}")
        if depth == 0:
            toks.append(Tok("NEWLINE", None, ln, n + 1))
    while len(indents) > 1:
        indents.pop()
        toks.append(Tok("DEDENT", 0, len(lines), 0))
    toks.append(Tok("EOF", None, len(lines) + 1, 0))
    return toks
