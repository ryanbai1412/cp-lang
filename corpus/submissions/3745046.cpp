#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

struct DSU {
	vector<int> p;

	DSU(int n): p(n, -1) {}
	
	int find(int u) {
		return p[u] < 0 ? u : p[u] = find(p[u]);
	}
	
	bool merge(int u, int v) {
		u = find(u);
		v = find(v);
		if(u == v) {
			return false;
		}
		if(p[u] < p[v]) {
			swap(u, v);
		}
		p[u] = v;
		return true;
	}
};

// so I can get 60 points by doing this?
void mst() {
	int n, m;
	cin >> n >> m;
	for(int i = 0; i < n; i++) {
		int x;
		cin >> x;
		assert(x >= n - 1);
	}
	tuple<int, int, int, int> edges[m];
	for(int i = 0; i < m; i++) {
		auto &[u, v, w, ii] = edges[i];
		cin >> u >> v >> w;
		u--;
		v--;
		ii = i + 1;
	}
	sort(edges, edges + m, [](const auto &a, const auto &b) -> bool {
		return get<2>(a) > get<2>(b);
	});
	DSU dsu(n);
	long ans = 0;
	vector<int> cans;
	for(auto &[u, v, w, i]: edges) {
		if(dsu.merge(u, v)) {
			ans += w;
			cans.push_back(i);
		}
	}
	cout << ans << endl;
	for(auto &a: cans) {
		cout << a << endl;
	}
}

void solve() {
	int t;
	cin >> t;
	if(t <= 6) {
		mst();
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}