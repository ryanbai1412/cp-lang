// corpus-written reference for WC 2022 P1 (no accepted C++), after the official solution:
// read (A)B as a binary-tree node with left A and right B; operations 1-4 then act on one node
// of the right chain. s1 goes to a right chain by operations 1-3 worked up from the chain's foot
// (one delete and one insert at the end); s2 goes to a right chain by inverse operation 4 from
// the root, with one leaf added at the foot so it always applies, and that path runs backwards.
// At most 3n operations; x is 2 (m - size) as every operated node's subtree is a suffix.
#include <bits/stdc++.h>
using namespace std;

struct Tree {
	vector<int> l, r, par, sz;
	int root = 0, m = 0;
	explicit Tree(const string& s) {
		int n = s.size() / 2;
		l.assign(n + 2, 0), r.assign(n + 2, 0), par.assign(n + 2, 0), sz.assign(n + 2, 0);
		vector<int> open, order;
		int prev = 0;
		bool prev_open = false;
		for (char c : s) {
			if (c == '(') {
				int v = ++m;
				order.push_back(v);
				if (prev) (prev_open ? l[prev] : r[prev]) = v, par[v] = prev;
				else root = v;
				open.push_back(v), prev = v, prev_open = true;
			} else {
				prev = open.back(), open.pop_back(), prev_open = false;
			}
		}
		for (int i = order.size() - 1; i >= 0; i--) pull(order[i]);
	}
	void pull(int v) { sz[v] = 1 + sz[l[v]] + sz[r[v]]; }
	void set_l(int v, int c) { l[v] = c; if (c) par[c] = v; }
	void set_r(int v, int c) { r[v] = c; if (c) par[c] = v; }
	int at(int v) const { return 2 * (m - sz[v]); }
};

int main() {
	int id, t;
	scanf("%d %d", &id, &t);
	while (t--) {
		int n, k;
		static char buf[2][200005];
		scanf("%d %d %200002s %200002s", &n, &k, buf[0], buf[1]);
		vector<pair<int, int>> ops;
		Tree a(buf[0]);
		int cur = a.root;
		while (a.r[cur]) cur = a.r[cur];
		while (true) {
			int y = a.l[cur];
			if (!y) {
				if (cur == a.root) break;
				cur = a.par[cur];
				continue;
			}
			if (a.l[y]) {  // op 1: X(Y(Z, C), D) -> X(Z, Y(C, D))
				ops.push_back({1, a.at(cur)});
				int z = a.l[y], c = a.r[y], d = a.r[cur];
				a.set_l(cur, z), a.set_r(y, d), a.set_l(y, c), a.set_r(cur, y), a.pull(y);
				cur = y;
			} else if (a.r[y]) {  // op 2: X(Y(-, V(B, C)), D) -> X(Y(-, B), V(C, D))
				ops.push_back({2, a.at(cur)});
				int v = a.r[y], b = a.l[v], c = a.r[v], d = a.r[cur];
				a.set_r(y, b), a.set_l(v, c), a.set_r(v, d), a.set_l(cur, y), a.set_r(cur, v);
				a.pull(y), a.pull(v);
				cur = v;
			} else {
				if (cur == a.root) break;
				int p = a.par[cur];  // op 3 at p: P(A, X(U, D)) -> P(U(A, -), X(-, D)), U a leaf
				ops.push_back({3, a.at(p)});
				int u = y, A = a.l[p];
				a.set_l(u, A), a.set_r(u, 0), a.set_l(cur, 0), a.set_l(p, u);
				a.pull(u), a.pull(cur);
				cur = p;
			}
		}
		if (a.l[a.root]) ops.push_back({6, 1}), ops.push_back({5, 2 * (n - 1)});

		Tree b(buf[1]);
		int foot = b.root;
		while (b.r[foot]) foot = b.r[foot];
		int extra = n + 1;
		b.m = n + 1, b.sz[extra] = 1, b.set_r(foot, extra);
		for (int v = foot; v; v = b.par[v]) b.pull(v);
		vector<pair<int, int>> back;
		for (int v = b.root; v;) {
			if (!b.l[v]) { v = b.r[v]; continue; }
			back.push_back({4, b.at(v)});  // inverse op 4: X(Z(A, B), W) -> X(A, Z(B, W))
			int z = b.l[v], A = b.l[z], B = b.r[z], w = b.r[v];
			b.set_l(v, A), b.set_l(z, B), b.set_r(z, w), b.set_r(v, z), b.pull(z);
		}
		if (!back.empty()) {
			ops.push_back({5, 2 * n});
			ops.insert(ops.end(), back.rbegin(), back.rend());
			ops.push_back({6, 2 * n});
		}
		if ((int)ops.size() > k) {
			puts("-1");
			continue;
		}
		printf("%d\n", (int)ops.size());
		for (auto [op, x] : ops) printf("%d %d\n", op, x);
	}
}
