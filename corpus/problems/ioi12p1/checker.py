#!/usr/bin/env python3
"""Checker for IOI 2012 Pebbling Odometer, packaged as a program: the input is a subtask (1-5), the
output an odometer program. The program is parsed here by the statement's rules and must be within
the subtask's full-score program size; the C++ below runs it, as the IOI simulator does, on a
256 x 256 grid for each of that subtask's edge and random grids, within the full-score execution
length, and checks the end state. Built like a reference, so a run takes well under a second."""
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "scripts"))

# Full-score program size and execution length per subtask.
LIMITS = {1: (100, 1000), 2: (200, 2000), 3: (100, 200_000), 4: (200, 200_000), 5: (444, 44_400_000)}
LABEL = re.compile(r"[A-Za-z0-9_]{1,128}")
OPS = ["left", "right", "move", "get", "put", "halt", "jump", "border", "pebble"]

SIMULATOR = r"""
#include <bits/stdc++.h>
using namespace std;
const int N = 256;
typedef array<int, N * N> Grid;
int task;
long long cap, most;
vector<pair<int, int>> prog;

// Runs prog on g; returns false past cap steps, else leaves the end cell in (r, c).
bool run(Grid& g, int& r, int& c) {
	static const int dr[] = {-1, 0, 1, 0}, dc[] = {0, 1, 0, -1};
	r = c = 0;
	int d = 0, pc = 0, end = prog.size();
	long long steps = 0;
	while (pc < end) {
		if (++steps > cap) return false;
		auto [op, t] = prog[pc++];
		int nr = r + dr[d], nc = c + dc[d];
		bool inside = nr >= 0 && nr < N && nc >= 0 && nc < N;
		switch (op) {
		case 0: d = (d + 3) % 4; break;
		case 1: d = (d + 1) % 4; break;
		case 2: if (inside) r = nr, c = nc; break;
		case 3: if (g[r * N + c]) g[r * N + c]--; break;
		case 4: if (g[r * N + c] < 15) g[r * N + c]++; break;
		case 5: pc = end; break;
		case 6: pc = t; break;
		case 7: if (!inside) pc = t; break;
		case 8: if (g[r * N + c]) pc = t; break;
		}
	}
	most = max(most, steps);
	return true;
}

bool ok(const Grid& start, const Grid& g, int r, int c) {
	if (task <= 2) {
		int x = start[0], y = start[1];
		if (r != 0 || c != (x <= y ? 0 : 1)) return false;
		return task == 1 || (g[0] == x && g[1] == y);
	}
	if (task == 3) {
		int s = 0;
		for (int j = 0; j < N; j++) if (start[j]) s += j;
		return r == 0 && c == s / 2;
	}
	if (task == 4) {
		int total = accumulate(start.begin(), start.end(), 0);
		return g[0] == total && accumulate(g.begin() + 1, g.end(), 0) == 0;
	}
	return g == start && g[r * N + c] == *min_element(g.begin(), g.end());
}

int main() {
	int n;
	cin >> task >> cap >> n;
	prog.resize(n);
	for (auto& [op, t] : prog) cin >> op >> t;
	mt19937 rng(task);
	auto rnd = [&](int lo, int hi) { return (int)(rng() % (hi - lo + 1)) + lo; };
	vector<Grid> grids;
	auto grid = [&](vector<array<int, 3>> cells) {
		Grid g{};
		for (auto [r, c, p] : cells) g[r * N + c] = p;
		grids.push_back(g);
	};
	if (task <= 2) {
		for (int x = 0; x < 16; x++)
			for (int y = 0; y < 16; y++) grid({{0, 0, x}, {0, 1, y}});
	} else if (task == 3) {
		vector<pair<int, int>> xy = {{0, 2}, {0, 254}, {1, 255}, {253, 255}, {254, 0}, {255, 1}, {100, 102}, {0, 128}};
		while (xy.size() < 100) {
			int x = rnd(0, N - 1), y = rnd(0, N - 1);
			if (x != y && (x + y) % 2 == 0) xy.push_back({x, y});
		}
		for (auto [x, y] : xy) grid({{0, x, 1}, {0, y, 1}});
	} else if (task == 4) {
		auto line = [&](int r0, int c0, int dr, int dc) {
			vector<array<int, 3>> v;
			for (int i = 0; i < 15; i++) v.push_back({r0 + dr * i, c0 + dc * i, 1});
			return v;
		};
		grid({});
		grid({{0, 0, 1}});
		grid({{N - 1, N - 1, 1}});
		grid(line(0, N - 15, 0, 1));
		grid(line(N - 1, N - 15, 0, 1));
		grid(line(N - 1, 0, 0, 1));
		grid(line(N - 15, N - 1, 1, 0));
		grid(line(N - 15, 0, 1, 0));
		grid(line(N - 15, N - 15, 1, 1));
		for (int k = 0; k < 100; k++) {
			set<pair<int, int>> at;
			int want = k < 3 ? 15 : rnd(1, 15);
			while ((int)at.size() < want) at.insert({k % 2 ? rnd(N - 8, N - 1) : rnd(0, N - 1), rnd(0, N - 1)});
			vector<array<int, 3>> v;
			for (auto [r, c] : at) v.push_back({r, c, 1});
			grid(v);
		}
	} else {
		Grid g;
		g.fill(15), grids.push_back(g);
		g.fill(0), grids.push_back(g);
		for (auto& p : g) p = rnd(0, 15);
		grids.push_back(g);
		int lows[] = {3, 9, 14, 14, 7};
		int at[][2] = {{N - 1, 0}, {N - 1, N - 1}, {0, N - 1}, {N - 1, N - 2}, {128, 77}};
		for (int k = 0; k < 5; k++) {
			for (auto& p : g) p = rnd(lows[k] + 1, 15);
			g[at[k][0] * N + at[k][1]] = lows[k];
			grids.push_back(g);
		}
	}
	for (auto& start : grids) {
		Grid g = start;
		int r, c;
		if (!run(g, r, c)) {
			cerr << "execution length over " << cap << "\n";
			return 1;
		}
		if (!ok(start, g, r, c)) {
			cerr << "wrong end state on grid " << (&start - grids.data()) << "\n";
			return 1;
		}
	}
	cerr << "longest execution: " << most << " steps on " << grids.size() << " grids\n";
}
"""


def parse(text):
    """The program as (opcode, target) pairs, or None when it breaks the statement's syntax."""
    ops, labels, uses = [], {}, []
    for line in text.split("\n"):
        words = line.split("#", 1)[0].split()
        if not words:
            continue
        if len(words) == 1 and words[0].endswith(":"):
            name = words[0][:-1]
            if not LABEL.fullmatch(name) or name in labels:
                return None
            labels[name] = len(ops)
        elif len(words) == 1 and words[0] in OPS[:6]:
            ops.append((OPS.index(words[0]), None))
        elif len(words) == 2 and words[0] in OPS[6:] and LABEL.fullmatch(words[1]):
            ops.append((OPS.index(words[0]), words[1]))
            uses.append(words[1])
        else:
            return None
    if any(u not in labels for u in uses):
        return None
    return [(op, labels[t] if t else 0) for op, t in ops]


def main():
    task = int(open(sys.argv[1]).read().split()[0])
    raw = open(sys.argv[2], "rb").read()
    prog = parse(raw.decode("utf-8", "replace")) if len(raw) <= 5 << 20 else None
    size, cap = LIMITS[task]
    if prog is None:
        sys.exit("not a valid odometer program")
    if len(prog) > size:
        sys.exit(f"program size {len(prog)} over {size}")
    import corpus
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "simulator.cpp")
        with open(src, "w") as f:
            f.write(SIMULATOR)
        exe = corpus.binary(src)
    data = f"{task} {cap} {len(prog)}\n" + "".join(f"{op} {t}\n" for op, t in prog)
    r = subprocess.run([exe], input=data, capture_output=True, text=True)
    sys.stderr.write(f"program size {len(prog)}; {r.stderr}")
    sys.exit(1 if r.returncode else 0)


if __name__ == "__main__":
    main()
