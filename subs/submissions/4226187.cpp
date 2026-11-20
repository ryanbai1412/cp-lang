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

const int maxn = 1e6 + 5;

int rep[maxn];
vector<pair<int, int>> graph[maxn];

void dfs(int u, int r, int p = -1) {
	rep[u] = r;
	for (auto& [v, i] : graph[u]) {
		if (v != p) {
			dfs(v, r ^ (1 << i), u);
		}
	}
}

void solve() {
	int n;
	cin >> n;
	for (int i = 0; i < n; i++) {
		int u, v;
		cin >> u >> v;
		graph[u].emplace_back(v, i);
		graph[v].emplace_back(u, i);
	}
	dfs(0, 0);
	cout << (1 << (n - 1)) << endl;
	for (int i = 0; i < (1 << n); i++) {
		if (__builtin_popcount(i) % 2 == 0) {
			for (int j = 0; j <= n; j++) {
				cout << (rep[j] ^ i) << " \n"[j == n];
			}
		}
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