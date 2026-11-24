#!/usr/bin/env python3
"""Checker for WC 2022 P1: the operation sequence is not unique. A case the answer calls -1 must
be -1; otherwise m <= k operations, each one valid at its x, at most two each of operations 5
and 6 (unless the test point id is 7, 8 or 14 to 16), must turn s1 into s2. Up to 3e5 operations
on 2e5 brackets need a balanced tree, so the rules run as the C++ below, built like a reference."""
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "scripts"))

VALIDATOR = r"""
#include <bits/stdc++.h>
using namespace std;
// Implicit treap over brackets (+1 / -1) with sums and minimum prefix sums.
struct Treap {
	vector<int> l{0}, r{0}, pri{0}, sz{0}, val{0}, sum{0}, mn{0};
	mt19937 rng{12345};
	int root = 0;
	int make(int v) {
		l.push_back(0), r.push_back(0), pri.push_back(rng()), sz.push_back(1);
		val.push_back(v), sum.push_back(v), mn.push_back(v);
		return val.size() - 1;
	}
	void pull(int t) {
		sz[t] = sz[l[t]] + 1 + sz[r[t]];
		sum[t] = sum[l[t]] + val[t] + sum[r[t]];
		int m = val[t] + sum[l[t]];
		if (l[t]) m = min(m, mn[l[t]]);
		if (r[t]) m = min(m, sum[l[t]] + val[t] + mn[r[t]]);
		mn[t] = m;
	}
	int merge(int a, int b) {
		if (!a || !b) return a | b;
		if (pri[a] > pri[b]) { r[a] = merge(r[a], b), pull(a); return a; }
		l[b] = merge(a, l[b]), pull(b);
		return b;
	}
	void split(int t, int k, int& a, int& b) {  // a: first k
		if (!t) { a = b = 0; return; }
		if (sz[l[t]] < k) split(r[t], k - sz[l[t]] - 1, r[t], b), a = t;
		else split(l[t], k, a, l[t]), b = t;
		pull(t);
	}
	int size() const { return sz[root]; }
	int at(int i) {
		int t = root;
		while (true) {
			if (i < sz[l[t]]) t = l[t];
			else if (i == sz[l[t]]) return val[t];
			else i -= sz[l[t]] + 1, t = r[t];
		}
	}
	int first_reach(int t, int target) {  // first index whose prefix sum is target (< 0), or -1
		if (!t || mn[t] > target) return -1;
		int base = 0, idx = 0;
		while (true) {
			if (l[t] && base + mn[l[t]] <= target) { t = l[t]; continue; }
			int here = base + sum[l[t]] + val[t];
			if (here <= target) return idx + sz[l[t]];
			base = here, idx += sz[l[t]] + 1, t = r[t];
		}
	}
	int match(int i) {  // index of the ')' closing the '(' at i, or -1
		if (i < 0 || i >= size() || at(i) != 1) return -1;
		int a, b;
		split(root, i + 1, a, b);
		int j = first_reach(b, -1);
		root = merge(a, b);
		return j < 0 ? -1 : i + 1 + j;
	}
	void insert(int i, int v) {
		int a, b;
		split(root, i, a, b);
		root = merge(merge(a, make(v)), b);
	}
	void erase(int i) {
		int a, b, c;
		split(root, i, a, b), split(b, 1, b, c);
		root = merge(a, c);
	}
	void out(int t, string& s) {
		if (!t) return;
		out(l[t], s), s += val[t] > 0 ? '(' : ')', out(r[t], s);
	}
};
int main(int argc, char** argv) {
	ifstream in(argv[1]), outf(argv[2]), ans(argv[3]);
	auto num = [](ifstream& f, long long& x) { string w; if (!(f >> w)) return false;
		try { size_t p; x = stoll(w, &p); return p == w.size(); } catch (...) { return false; } };
	auto fail = [](const string& m) { fprintf(stderr, "%s\n", m.c_str()); exit(1); };
	int id, t;
	in >> id >> t;
	bool free56 = id == 7 || id == 8 || (id >= 14 && id <= 16);
	for (int c = 1; c <= t; c++) {
		long long n, k, m, want, x;
		string s1, s2;
		in >> n >> k >> s1 >> s2;
		if (!num(ans, want)) fail("bad answer file");
		for (long long i = 0; i < 2 * max(want, 0LL); i++) num(ans, x);
		if (!num(outf, m)) fail("case " + to_string(c) + ": no m");
		if (want == -1 || m == -1) {
			if (m != want) fail("case " + to_string(c) + ": -1 disagrees with the answer");
			continue;
		}
		if (m < 0 || m > k) fail("case " + to_string(c) + ": m out of [0, k]");
		Treap tr;
		for (char ch : s1) tr.root = tr.merge(tr.root, tr.make(ch == '(' ? 1 : -1));
		int c5 = 0, c6 = 0;
		for (long long i = 0; i < m; i++) {
			long long op;
			if (!num(outf, op) || !num(outf, x)) fail("case " + to_string(c) + ": short output");
			string where = "case " + to_string(c) + " op " + to_string(i + 1) + ": ";
			int len = tr.size();
			if (x < 0 || x > len) fail(where + "x out of range");
			auto open = [&](long long p) { return p >= 0 && p < tr.size() && tr.at(p) == 1; };
			if (op == 5) { tr.insert(x, -1), tr.insert(x, 1), c5++; continue; }
			if (op == 6) {
				if (x + 1 >= len || tr.at(x) != 1 || tr.at(x + 1) != -1) fail(where + "no () at x");
				tr.erase(x), tr.erase(x), c6++;
				continue;
			}
			bool ok = false;
			if (op == 1 && open(x + 1) && open(x + 2)) {
				int a = tr.match(x + 2), b = tr.match(x + 1), cc = tr.match(x);
				if (a >= 0 && b >= 0 && cc >= 0) ok = true, tr.erase(x), tr.insert(b, 1);
			} else if (op == 2 && open(x + 1)) {
				int a = tr.match(x + 1), cc = tr.match(x);
				if (a >= 0 && cc >= 0 && open(a + 1)) {
					int b = tr.match(a + 1);
					if (b >= 0 && b < cc) ok = true, tr.erase(a + 1), tr.insert(b, 1);
				}
			} else if (op == 3) {
				int a = tr.match(x);
				if (a >= 0 && open(a + 1) && open(a + 2)) {
					int b = tr.match(a + 2), cc = tr.match(a + 1);
					if (b >= 0 && cc >= 0) ok = true, tr.erase(a + 1), tr.erase(a + 1), tr.insert(x, 1), tr.insert(b, 1);
				}
			} else if (op == 4) {
				int a = tr.match(x);
				if (a >= 0 && open(a + 1)) {
					int b = tr.match(a + 1);
					if (b >= 0 && open(b + 1) && tr.match(b + 1) >= 0) ok = true, tr.erase(a + 1), tr.insert(x, 1);
				}
			}
			if (!ok) fail(where + "operation " + to_string(op) + " does not apply at x");
		}
		if (!free56 && (c5 > 2 || c6 > 2)) fail("case " + to_string(c) + ": operations 5/6 used more than twice");
		string got;
		tr.out(tr.root, got);
		if (got != s2) fail("case " + to_string(c) + ": the result is not s2");
	}
	long long extra;
	if (num(outf, extra)) fail("trailing output");
}
"""


def main():
    if len(sys.argv) != 4:
        sys.stderr.write("usage: checker.py INPUT OUTPUT ANSWER\n")
        sys.exit(2)
    import corpus
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "validator.cpp")
        with open(src, "w") as f:
            f.write(VALIDATOR)
        exe = corpus.binary(src)
    r = subprocess.run([exe, *sys.argv[1:]], capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(r.stderr or "rejected\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
