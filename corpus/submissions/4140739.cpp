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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

struct DSU {
	vector<int> p;

	explicit DSU(int n) : p(n, -1) {}

	int find(int u) {
		return p[u] == -1 ? u : (p[u] = find(p[u]));
	}

	void merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u != v) {
			p[u] = v;
		}
	}

	int size(int u) {
		return -p[find(u)];
	}
};

const int maxn = 5e5 + 5;

int n, m, x, q, edges[maxn][2];

template <typename T>
bool solve() {
	T pq;
	DSU dsu(n);
	for (int i = 0; i < q; i++) {
		dsu.merge(edges[i][0], edges[i][1]);
	}
	for (int i = 0; i < n; i++) {
		if (dsu.find(i) == i) {
			pq.emplace(dsu.size(i), i);
		}
	}
	if (sz(pq) < x) {
		return false;
	}
	while (sz(pq) > x) {
		auto [_a, u] = pq.top();
		pq.pop();
		auto [_b, v] = pq.top();
		pq.pop();
		dsu.merge(u, v);
		pq.emplace(dsu.size(u), dsu.find(u));
	}
	vector<pair<int, int>> creps;
	while (sz(pq)) {
		creps.push_back(pq.top());
		pq.pop();
	}
	int cconv[n];
	for (int i = 0; i < sz(creps); i++) {
		cconv[creps[i].second] = i;
	}
	vector<vector<int>> comps(sz(creps));
	for (int i = 0; i < n; i++) {
		comps[cconv[dsu.find(i)]].push_back(i);
	}
	vector<pair<int, int>> cans;
	for (auto& a : comps) {
		if (sz(a) > 1) {
			cans.emplace_back(a.back(), a[0]);
			for (int i = 0; i < sz(a) - 1; i++) {
				cans.emplace_back(a[i], a[i + 1]);
			}
		}
	}
	if (sz(cans) > m) {
		return false;
	}
	int cn = sz(comps);
	for (int i = 0; i < cn; i++) {
		if (sz(cans) == m) {
			break;
		}
		int cm = sz(comps[i]);
		for (int a = 0; a < cm; a++) {
			if (sz(cans) == m) {
				break;
			}
			for (int b = 0; b < cm; b++) {
				if (sz(cans) == m) {
					break;
				}
				if (a != b && (a + 1) % cm != b) {
					cans.emplace_back(comps[i][a], comps[i][b]);
				}
			}
		}
	}
	for (int i = 0; i < cn; i++) {
		if (sz(cans) == m) {
			break;
		}
		for (int j = i + 1; j < cn; j++) {
			if (sz(cans) == m) {
				break;
			}
			for (auto& a : comps[i]) {
				if (sz(cans) == m) {
					break;
				}
				for (auto& b : comps[j]) {
					if (sz(cans) == m) {
						break;
					}
					cans.emplace_back(a, b);
				}
			}
		}
	}
	if (sz(cans) != m) {
		return false;
	}
	sort(begin(cans), end(cans));
	assert(unique(begin(cans), end(cans)) == cans.end());
	for (auto& [u, v] : cans) {
		assert(0 <= u && u < n && 0 <= v && v < n && u != v);
		cout << u + 1 << " " << v + 1 << endl;
	}
	return true;
}

void solve() {
	cin >> n >> m >> x >> q;
	for (int i = 0; i < q; i++) {
		auto& [u, v] = edges[i];
		cin >> u >> v;
		u--;
		v--;
	}
	if (!solve<priority_queue<pair<int, int>>>()) {
		cout << -1 << endl;
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