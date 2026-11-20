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

const int maxn = 5e4;

vector<pair<int, int>> graph[maxn];

void pdfs(int u) {
	for (auto& [v, w] : graph[u]) {
		graph[v].erase(
			find(begin(graph[v]), end(graph[v]), pair<int, int> {u, w}));
		pdfs(v);
	}
}

struct Solver {
	int k, ans = -1;

	explicit Solver(int k) : k(k) {}

	int dfs(int u) {
		multiset<int> vals;
		for (auto& [v, w] : graph[u]) {
			int cur = dfs(v) + w;
			if (cur < k) {
				vals.insert(cur);
			} else {
				ans++;
			}
		}
		int ret = 0;
		while(sz(vals)) {
			int cur = *vals.begin();
			vals.erase(vals.begin());
			auto it = vals.lower_bound(k - cur);
			if (it != vals.end()) {
				vals.erase(it);
				ans++;
			} else {
				ret = cur;
			}
		}
		return ret;
	}

	int solve() {
		if (ans == -1) {
			ans = 0;
			dfs(0);
		}
		return ans;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n - 1; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	pdfs(0);
	int l = 0, r = 1e9;
	while (l < r) {
		int mid = (l + r + 1) / 2;
		Solver s(mid);
		if (s.solve() >= m) {
			l = mid;
		} else {
			r = mid - 1;
		}
	}
	cout << l << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}