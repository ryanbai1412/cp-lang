#!/usr/bin/env python3
"""Reduce spec/std/*.cp to reference docs: every fn body becomes `todo`.

The spec is signatures, doc comments and costs; implementations live in
stdlib/. Only fn bodies change: headers, docs, traits, structs and bodiless
trait methods are kept as written. `lends` fns keep their bodies, since the
checker needs the `lend place` to type the place they return.

Usage: scripts/spec-stub.py [--check] [FILE...]   (default spec/std/*.cp)
--check changes nothing and exits 1 if any fn still has a body.
"""
import glob, os, re, sys

FN = re.compile(r"^(\s*)(?:(?:@\S+|pub|extern)\s+)*fn\b")


def code(line):
    """The line without its comment and with string contents blanked."""
    out, q, i = [], None, 0
    while i < len(line):
        c = line[i]
        if q:
            out.append(" " if c != q else c)
            if c == "\\":
                out.append(" "); i += 1
            elif c == q:
                q = None
        elif c in "\"'":
            q = c; out.append(c)
        elif line.startswith("//", i):
            break
        else:
            out.append(c)
        i += 1
    return "".join(out)


def header_end(lines, i):
    """(last header line, column of the body's `:` or `=`, kind) or None."""
    depth, seen_paren = 0, False
    for j in range(i, min(i + 40, len(lines))):
        s = code(lines[j])
        k = 0
        while k < len(s):
            c = s[k]
            if c in "([{":
                depth += 1
                seen_paren |= c == "("
            elif c in ")]}":
                depth -= 1
            elif not seen_paren and c == "<":
                depth += 1
            elif not seen_paren and c == ">" and s[k - 1:k] != "-":
                depth -= 1
            elif depth == 0 and c == "=" and s[k - 1:k] not in ("=", "<", ">", "!", ".") \
                    and s[k + 1:k + 2] not in ("=", ">"):
                return j, k, "="
            elif depth == 0 and c == ":" and s[k + 1:k + 2] != ":" and s[k - 1:k] != ":":
                return j, k, ":"
            k += 1
        if depth == 0 and j > i and not s.strip():
            return None
        if depth == 0 and not s.rstrip().endswith((",", "(", "->")):
            nxt = lines[j + 1] if j + 1 < len(lines) else ""
            if not code(nxt).lstrip().startswith(("->", ")")):
                return None
    return None


def brackets(s):
    return sum(s.count(c) for c in "([{") - sum(s.count(c) for c in ")]}")


def indent(line):
    return len(line) - len(line.lstrip())


def stub(lines):
    out, i, changed = [], 0, 0
    while i < len(lines):
        m = FN.match(lines[i])
        end = header_end(lines, i) if m else None
        if not end:
            out.append(lines[i]); i += 1; continue
        j, col, kind = end
        base = indent(lines[i])
        head = lines[i:j] + [lines[j][:col].rstrip()]
        rest = code(lines[j])[col + 1:].strip()
        k, depth = j + 1, brackets(rest)
        while k < len(lines) and (not lines[k].strip() or indent(lines[k]) > base or depth > 0):
            depth += brackets(code(lines[k]))
            k += 1
        while k > j + 1 and not lines[k - 1].strip():
            k -= 1
        lends = " lends " in " ".join(code(l) for l in head)
        if lends or (rest == "todo" and k == j + 1):
            out.extend(lines[i:k])
        else:
            out.extend(head[:-1] + [head[-1] + " = todo"])
            changed += 1
        i = k
    return out, changed


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if a != "--check"] or sorted(glob.glob(
        os.path.join(os.path.dirname(__file__), "..", "spec", "std", "*.cp")))
    total = 0
    for f in files:
        lines = open(f).read().split("\n")
        new, n = stub(lines)
        total += n
        if n and not check:
            open(f, "w").write("\n".join(new))
        if n:
            print(f"{os.path.relpath(f)}: {n} body(ies){' left' if check else ' stubbed'}")
    print(f"spec-stub: {total} fn body(ies) {'with bodies' if check else 'stubbed'}")
    return 1 if check and total else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
