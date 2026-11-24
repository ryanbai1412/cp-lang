// WC/NOI 2022 multi-provincial selection, Community: a max flow over "who comes right after whom".
// Message z right before w earns a point for w if w is louxia naming sender(z) (type L), and for
// z if z is loushang naming sender(w) (type U). Mutual pairs (a b loushang)(b a louxia) earn two
// and are paired first; the rest is a bipartite matching of next slots to previous slots through
// one hub per person and type. A matching cycle is all louxia or all loushang, and swapping one
// of its edges with the academic message every person has breaks it without losing a point.
#include <bits/stdc++.h>
using namespace std;

struct Dinic {
	struct E { int to, cap; };
	vector<E> e;
	vector<vector<int>> g;
	vector<int> lvl, it;
	Dinic(int n) : g(n), lvl(n), it(n) {}
	int add(int a, int b, int c) {
		g[a].push_back(e.size()), e.push_back({b, c});
		g[b].push_back(e.size()), e.push_back({a, 0});
		return e.size() - 2;
	}
	bool bfs(int s, int t) {
		fill(lvl.begin(), lvl.end(), -1);
		queue<int> q;
		q.push(s), lvl[s] = 0;
		while (!q.empty()) {
			int u = q.front();
			q.pop();
			for (int id : g[u])
				if (e[id].cap && lvl[e[id].to] < 0) lvl[e[id].to] = lvl[u] + 1, q.push(e[id].to);
		}
		return lvl[t] >= 0;
	}
	int dfs(int u, int t, int f) {
		if (u == t) return f;
		for (int& i = it[u]; i < (int)g[u].size(); i++) {
			E& x = e[g[u][i]];
			if (x.cap && lvl[x.to] == lvl[u] + 1) {
				int d = dfs(x.to, t, min(f, x.cap));
				if (d) {
					x.cap -= d, e[g[u][i] ^ 1].cap += d;
					return d;
				}
			}
		}
		return 0;
	}
	long long run(int s, int t) {
		long long f = 0;
		while (bfs(s, t)) {
			fill(it.begin(), it.end(), 0);
			while (int d = dfs(s, t, INT_MAX)) f += d;
		}
		return f;
	}
};

int main() {
	ios::sync_with_stdio(false), cin.tie(nullptr);
	int T;
	cin >> T;
	while (T--) {
		int n, m;
		cin >> n >> m;
		unordered_map<string, int> id;
		for (int i = 0; i < n; i++) {
			string s;
			cin >> s;
			id[s] = i;
		}
		// kind: 0 academic, 1 louxia (wants sender(prev) == men), 2 loushang (wants sender(next) == men)
		vector<int> snd(m), men(m, -1), kind(m, 0), acad(n, -1);
		for (int i = 0; i < m; i++) {
			string a, b, c;
			cin >> a >> b >> c;
			snd[i] = id[a];
			auto f = id.find(b);
			if (f != id.end() && (c == "louxia" || c == "loushang")) men[i] = f->second, kind[i] = c == "louxia" ? 1 : 2;
			else acad[snd[i]] = i;
		}
		vector<int> nxt(m, -1), prv(m, -1);
		auto link = [&](int a, int b) { nxt[a] = b, prv[b] = a; };
		{  // mutual pairs: (a b loushang) right before (b a louxia)
			map<pair<int, int>, vector<int>> wantL;
			for (int i = 0; i < m; i++)
				if (kind[i] == 1) wantL[{men[i], snd[i]}].push_back(i);
			for (int i = 0; i < m; i++)
				if (kind[i] == 2) {
					auto f = wantL.find({snd[i], men[i]});
					if (f != wantL.end() && !f->second.empty()) link(i, f->second.back()), f->second.pop_back();
				}
		}
		// nodes: S, T, next slot 2 + z, previous slot 2 + m + w, hub L 2 + 2m + p, hub U 2 + 2m + n + p
		int S = 0, Tn = 1, NX = 2, PV = 2 + m, HL = 2 + 2 * m, HU = 2 + 2 * m + n;
		Dinic d(2 + 2 * m + 2 * n);
		vector<int> eL(m, -1), eU(m, -1), inL(m, -1), inU(m, -1);
		for (int z = 0; z < m; z++) {
			if (nxt[z] < 0) {
				d.add(S, NX + z, 1);
				eL[z] = d.add(NX + z, HL + snd[z], 1);
				if (kind[z] == 2) eU[z] = d.add(NX + z, HU + men[z], 1);
			}
			if (prv[z] < 0) {
				d.add(PV + z, Tn, 1);
				if (kind[z] == 1) inL[z] = d.add(HL + men[z], PV + z, 1);
				inU[z] = d.add(HU + snd[z], PV + z, 1);
			}
		}
		d.run(S, Tn);
		vector<vector<int>> fromL(n), fromU(n);
		for (int z = 0; z < m; z++) {
			if (eL[z] >= 0 && !d.e[eL[z]].cap) fromL[snd[z]].push_back(z);
			if (eU[z] >= 0 && !d.e[eU[z]].cap) fromU[men[z]].push_back(z);
		}
		for (int w = 0; w < m; w++) {
			if (inL[w] >= 0 && !d.e[inL[w]].cap) link(fromL[men[w]].back(), w), fromL[men[w]].pop_back();
			if (inU[w] >= 0 && !d.e[inU[w]].cap) link(fromU[snd[w]].back(), w), fromU[snd[w]].pop_back();
		}
		// break cycles: all louxia (type L edges) or all loushang (type U edges)
		vector<int> seen(m, 0);
		for (int s = 0; s < m; s++) {
			if (seen[s]) continue;
			int u = s;
			while (prv[u] >= 0 && prv[u] != s && !seen[u]) u = prv[u];
			if (prv[u] != s || seen[u]) {  // a path: mark it
				for (int v = u; v >= 0 && !seen[v]; v = nxt[v]) seen[v] = 1;
				continue;
			}
			for (int v = s; !seen[v]; v = nxt[v]) seen[v] = 1;
			int z = s, w = nxt[s];
			if (kind[w] == 1) {  // type L: z -> w, give w the academic q of sender(z)
				int q = acad[snd[z]], v = nxt[q];
				nxt[z] = -1, prv[w] = -1;
				if (v >= 0) prv[v] = -1, nxt[q] = -1, link(z, v);
				link(q, w);
			} else {  // type U: z -> w, give z the academic q of sender(w)
				int q = acad[snd[w]], u2 = prv[q];
				nxt[z] = -1, prv[w] = -1;
				if (u2 >= 0) nxt[u2] = -1, prv[q] = -1, link(u2, w);
				link(z, q);
			}
		}
		vector<int> order;
		for (int s = 0; s < m; s++)
			if (prv[s] < 0)
				for (int v = s; v >= 0; v = nxt[v]) order.push_back(v);
		assert((int)order.size() == m);
		int pts = 0;
		for (int i = 0; i < m; i++) {
			int z = order[i];
			if (kind[z] == 1 && i > 0 && snd[order[i - 1]] == men[z]) pts++;
			if (kind[z] == 2 && i + 1 < m && snd[order[i + 1]] == men[z]) pts++;
		}
		cout << pts << '\n';
		for (int i = 0; i < m; i++) cout << order[i] + 1 << " \n"[i + 1 == m];
	}
}
