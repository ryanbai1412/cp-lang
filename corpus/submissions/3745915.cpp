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

	bool conn(int u, int v) {
		return find(u) == find(v);
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

	void clear() {
		fill(begin(p), end(p), -1);
	}
};

void mst() {
	int n, m;
	cin >> n >> m;
	int deg[n];
	for(int i = 0; i < n; i++) {
		cin >> deg[i];
	}
	array<int, 4> edges[m];
	for(int i = 0; i < m; i++) {
		auto &[u, v, w, ii] = edges[i];
		cin >> u >> v >> w;
		u--;
		v--;
		ii = i + 1;
	}
	sort(edges, edges + m, [](const auto &a, const auto &b) -> bool {
		return a[2] > b[2];
	});
	DSU dsu(n);
	long ans = 0;
	vector<int> cans;
	for(auto &[u, v, w, i]: edges) {
		if(deg[u] && deg[v] && dsu.merge(u, v)) {
			deg[u]--;
			deg[v]--;
			ans += w;
			cans.push_back(i);
		}
	}
	assert(sz(cans) == n - 1);
	cout << ans << endl;
	for(auto &a: cans) {
		cout << a << endl;
	}
}

void rmst(int tc) {
	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	int n, m;
	cin >> n >> m;
	int ideg[n];
	for(int i = 0; i < n; i++) {
		cin >> ideg[i];
	}
	array<int, 4> edges[m];
	for(int i = 0; i < m; i++) {
		auto &[u, v, w, ii] = edges[i];
		cin >> u >> v >> w;
		u--;
		v--;
		ii = i + 1;
	}
	sort(edges, edges + m, [](const auto &a, const auto &b) -> bool {
		return a[2] > b[2];
	});
	long bans = 0;
	vector<int> xans;
	DSU dsu(n);
	auto calc = [&]() -> bool {
		dsu.clear();
		long ans = 0;
		vector<int> cans;
		int deg[n];
		memcpy(deg, ideg, sizeof(deg));
		for(int i = 0; i < m; i++) {
			auto &[u, v, w, _] = edges[i];
			if(deg[u] && deg[v] && dsu.merge(u, v)) {
				deg[u]--;
				deg[v]--;
				ans += w;
				cans.push_back(i);
			}
		}
		if(sz(cans) != n - 1) {
			return false;
		}
		if(ans > bans) {
#ifdef LOCAL
			cerr << "\rbans: " << (bans = ans);
			ofstream fout("2005/out/" + to_string(tc) + ".txt");
			fout << ans << endl;
			for(auto &a: cans) {
				fout << edges[a][3] << endl;
			}
#else
			xans = cans;
#endif
			return true;
		}
		return false;
	};
	calc();
	auto duration = chrono::seconds(2);
#ifdef LOCAL
	duration = chrono::hours(1412);
#endif
	for(auto start = chrono::steady_clock::now(); chrono::steady_clock::now() - start <= duration ; ) {
		int u, v;
		do {
			u = cowng() % m;
			v = cowng() % m;
		}while(u == v);
		swap(edges[u], edges[v]);
		if(!calc()) {
			swap(edges[u], edges[v]);
		}
	}
	cout << bans << endl;
	for(auto &a: xans) {
		cout << edges[a][3] << endl;
	}
}

void solve() {
	int t;
	cin >> t;
	if(t <= 3) {
		mst();
	}else {
		rmst(t);
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