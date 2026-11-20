#!/usr/bin/env python3
"""Dual LOC counter for Scout Batch 42 (Hard Mixed Corpus Classics).

Counts non-blank, non-comment lines for every problem:
  raw       entire original C++ submission in corpus/submissions/
  stripped  original C++ minus contest template header (through `#define sz(`),
            `main()` stream/driver boilerplate, and reusable data structures /
            dead template pastes (`struct IO`, `struct DS` in mccc6s3;
            `struct BIT1/BIT2/RBIT` in occ19g4; `pb_ds ost` typedef in mccc5j3;
            unused `struct DSU` in mccc4j4)
  v2        <problem>.cp (locked CP Lang v2 spec)
  today     today/<problem>.cp (runnable via `proto/cp.py lib-run`)
"""
import os, re

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

# (problem, submission_id, extra stripped 1-based inclusive line ranges for reusable DS / dead code)
SPECS = [
    ("mccc6s3", "6103125", [(28, 45), (47, 85)]),  # struct IO + struct DS
    ("occ19g4", "3578469", [(17, 75)]),            # struct BIT1, BIT2, RBIT
    ("mccc5j5", "5333196", []),                    # problem-specific XOR trie kept
    ("mccc5j3", "5333116", [(32, 38)]),            # pb_ds ost typedef
    ("mccc4j4", "4264934", [(32, 54)]),            # unused struct DSU
    ("mccc3j4", "4223440", []),
    ("mccc3j5", "4223449", []),
    ("mccc5j4", "5333145", []),
    ("mccc3j3", "4223424", []),
    ("mccc4j3", "4264907", []),
]

BOILERPLATE = re.compile(
    r"^(int main\(|cin\.tie|ios_base::sync_with_stdio|cin\.exceptions|#ifdef LOCAL|freopen|#endif|int t = 1;|for \(int _ = 1|dbg\(_\)|solve\(\);|\})$"
)


def is_code(line):
    s = line.strip()
    return bool(s) and not s.startswith("//")


def read_lines(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read().splitlines()


def count_stripped(cpp_lines, extra_ranges):
    cut = max(i for i, l in enumerate(cpp_lines) if l.startswith("#define sz(")) + 1
    # Also locate `int main(` so we strip the generic main() driver cleanly.
    main_idx = next(i for i, l in enumerate(cpp_lines) if l.strip().startswith("int main("))
    kept = []
    for idx0, line in enumerate(cpp_lines):
        ln = idx0 + 1
        if idx0 < cut or idx0 >= main_idx:
            continue
        if any(lo <= ln <= hi for lo, hi in extra_ranges):
            continue
        if is_code(line):
            kept.append(line)
    return len(kept)


def main():
    tot = [0, 0, 0, 0]
    print(f"{'problem':10} {'sub':>8} {'raw':>5} {'stripped':>8} {'v2':>4} {'today':>5} {'v2/raw':>8} {'v2/str':>8}")
    for prob, sub, extra in SPECS:
        cpp = read_lines(os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp"))
        raw = sum(1 for l in cpp if is_code(l))
        stripped = count_stripped(cpp, extra)
        v2 = sum(1 for l in read_lines(os.path.join(HERE, f"{prob}.cp")) if is_code(l))
        today = sum(1 for l in read_lines(os.path.join(HERE, "today", f"{prob}.cp")) if is_code(l))
        tot = [a + b for a, b in zip(tot, [raw, stripped, v2, today])]
        r_red = 100.0 * (1 - v2 / raw)
        s_red = 100.0 * (1 - v2 / stripped)
        print(f"{prob:10} {sub:>8} {raw:5} {stripped:8} {v2:4} {today:5} {-r_red:+7.1f}% {-s_red:+7.1f}%")
    print("-" * 64)
    print(
        f"{'total':10} {'':>8} {tot[0]:5} {tot[1]:8} {tot[2]:4} {tot[3]:5} "
        f"{-(100*(1-tot[2]/tot[0])):+7.1f}% {-(100*(1-tot[2]/tot[1])):+7.1f}%"
    )
    print(
        f"today vs raw: {-(100*(1-tot[3]/tot[0])):+.1f}%   "
        f"today vs stripped: {-(100*(1-tot[3]/tot[1])):+.1f}%"
    )


if __name__ == "__main__":
    main()
