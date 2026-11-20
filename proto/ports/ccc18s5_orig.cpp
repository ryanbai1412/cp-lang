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
#define dbg(...)                                           \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
		 << ": ";                                          \
	dbgh(__VA_ARGS__)
#else
#define cerr   \
	if (false) \
	cerr
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct DSU {
	int comps;
	vector<int> p;
	
	explicit DSU(int n): comps(n), p(n, -1) {}
	
	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}
	
	bool merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return false;
		}
		comps--;
		if (p[u] < p[v]) {
			swap(u, v);
		}
		p[v] += p[u];
		p[u] = v;
		return true;
	}
};

void solve() {
	int n, m, p, q;
	cin >> n >> m >> p >> q;
	array<int, 3> hori[p], vert[q];
	for (auto& [u, v, w] : hori) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	for (auto& [u, v, w] : vert) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	vector<array<int, 4>> edges;
	for (auto& [u, v, w] : hori) {
		edges.push_back({w, 0, u, v});
	}
	for (auto& [u, v, w] : vert) {
		edges.push_back({w, 1, u, v});
	}
	sort(begin(edges), end(edges));
	DSU hori_dsu(m), vert_dsu(n);
	long ans = 0;
	for (auto& [w, is_vert, u, v] : edges) {
		if (is_vert) {
			if (vert_dsu.merge(u, v)) {
				dbg(u, v, w, hori_dsu.comps);
				ans += long(w) * hori_dsu.comps;
			}
		} else {
			if (hori_dsu.merge(u, v)) {
				dbg(u, v, w, vert_dsu.comps);
				ans += long(w) * vert_dsu.comps;
			}
		}
	}
	dbg(ans);
	ans *= -1;
	for (auto& [_u, _v, w] : hori) {
		ans += long(w) * n;
	}
	for (auto& [_u, _v, w] : vert) {
		ans += long(w) * m;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}