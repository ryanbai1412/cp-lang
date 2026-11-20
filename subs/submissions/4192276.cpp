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

const long mod = 998244353;

struct DSU {
	vector<int> p;

	explicit DSU(int n) : p(n, -1) {}

	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}

	bool merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return false;
		}
		if (p[u] < p[v]) {
			swap(u, v);
		}
		p[v] += p[u];
		p[u] = v;
		return true;
	}
};

void solve() {
	long n;
	int a, b;
	cin >> n >> a >> b;
	if (a == 2 && b == 0) {
		long d1, w1, d2, w2;
		cin >> d1 >> w1 >> d2 >> w2;
		if (w1 > w2) {
			swap(w1, w2);
		}
		long comps = max(d1 + d2 - n, gcd(d1, d2));
		long ans = (n - d1) % mod * w1;
		ans += (d1 - comps) % mod * w2;
		cout << ans % mod << endl;
	} else if (n <= long(2e5) && a <= 10) {
		vector<tuple<long, int, int>> edges;
		for (int i = 0; i < a; i++) {
			int d, x;
			cin >> d >> x;
			for (int j = 0; j + d < n; j++) {
				edges.emplace_back(x, j, j + d);
			}
		}
		for (int i = 0; i < b; i++) {
			int u, v, w;
			cin >> u >> v >> w;
			edges.emplace_back(w, u, v);
		}
		sort(begin(edges), end(edges));
		long ans = 0;
		DSU dsu((int(n)));
		for (auto& [w, u, v] : edges) {
			if (dsu.merge(u, v)) {
				ans += w;
			}
		}
		cout << ans % mod << endl;
	}
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