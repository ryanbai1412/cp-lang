#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

long c2(long x) {
	return x * (x - 1) / 2;
}

struct DSU {
	long ans;
	vector<int> p;
	vector<vector<int>> c;

	DSU(int n) : ans(0), p(n, -1), c(n) {
		for (int i = 0; i < n; i++) {
			c[i].push_back(i);
		}
	}

	int find(int u) {
		return p[u] == -1 ? u : (p[u] = find(p[u]));
	}

	vector<int> merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return {};
		}
		if (sz(c[u]) > sz(c[v])) {
			swap(u, v);
		}
		ans -= c2(sz(c[u])) + c2(sz(c[v]));
		p[u] = v;
		c[v].insert(c[v].end(), begin(c[u]), end(c[u]));
		ans += c2(sz(c[v]));
		return c[u];
	}
};

struct R {
	long ans;
	vector<pair<int, int>> changes;
};

struct Edge {
	int u, v, w;

	bool operator<(const Edge& e) const {
		return w < e.w;
	}
};

struct DS {
	DSU a;
	long bans, dup;
	vector<R> r;
	vector<int> bfind;
	vector<pair<int, int>> rep;
	map<pair<int, int>, int> m;

	DS(int n, const vector<Edge>& arr) : a(n), dup(0), bfind(n), rep(n), m {} {
		DSU b(n);
		for (int i = 0; i < n; i++) {
			bfind[i] = b.find(i);
		}
		for (auto& [u, v, _] : arr) {
			long cans = b.ans;
			auto c = b.merge(u, v);
			vector<pair<int, int>> ch;
			for (auto& x : c) {
				ch.emplace_back(x, bfind[x]);
				bfind[x] = b.find(x);
			}
			r.push_back({cans, ch});
		}
		bans = b.ans;
		for (int i = 0; i < n; i++) {
			rep[i] = {a.find(i), b.find(i)};
			m[rep[i]]++;
		}
	}

	long ans() const {
		return a.ans + bans - dup;
	}

	void update(const vector<int>& arr) {
		for (auto& x : arr) {
			{
				int& ax = m[rep[x]];
				dup -= c2(ax);
				dup += c2(--ax);
			}
			rep[x] = {a.find(x), bfind[x]};
			{
				int& bx = m[rep[x]];
				dup -= c2(bx);
				dup += c2(++bx);
			}
		}
	}

	void mergea(int u, int v) {
		update(a.merge(u, v));
	}

	void rollbackb() {
		bans = r.back().ans;
		vector<int> n;
		for (auto& [i, x] : r.back().changes) {
			n.push_back(i);
			bfind[i] = x;
		}
		update(n);
		r.pop_back();
	}
};

void solve() {
	int n, ma, mb;
	long k;
	cin >> n >> ma >> mb >> k;
	Edge arra[ma];
	vector<Edge> arrb(mb);
	for (auto& [u, v, w] : arra) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	for (auto& [u, v, w] : arrb) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	sort(arra, arra + ma);
	sort(begin(arrb), end(arrb));
	DS ds(n, arrb);
	int j = mb - 1;
	int ans = INT_MAX;
	auto upd = [&](int w) -> void {
		for (; j >= 0 && ds.ans() >= k; j--) {
			ds.rollbackb();
		}
		if (ds.ans() >= k) {
			ans = min(ans, w);
		}
		if (j < mb - 1) {
			ans = min(ans, w + arrb[j + 1].w);
		}
	};
	upd(0);
	for (auto& [u, v, w] : arra) {
		ds.mergea(u, v);
		upd(w);
	}

	if (ans == INT_MAX) {
		cout << -1 << endl;
	} else {
		cout << ans << endl;
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}