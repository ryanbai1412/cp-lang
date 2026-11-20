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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

bool on(int msk, int bit) {
	return (msk >> bit) & 1;
}

int clique(const vector<vector<bool>>& graph) {
	int n = sz(graph);
	int dp[1 << n];
	for (int i = (1 << n) - 1; i >= 0; i--) {
		dp[i] = __builtin_popcount(i);
		for (int j = 0; j < n; j++) {
			if (!on(i, j)) {
				for (int k = 0; k < n; k++) {
					if (on(i, k) && !graph[j][k]) {
						goto loop;
					}
				}
				dp[i] = max(dp[i], dp[i | (1 << j)]);
			loop:;
			}
		}
	}
	return dp[0];
}

void solve() {
	int n, k;
	cin >> n >> k;
	vector<int> graph[n];
	for (auto& a : graph) {
		int m;
		cin >> m;
		a.resize(m);
		for (auto& b : a) {
			cin >> b;
		}
		sort(begin(a), end(a));
	}
	bool del[n] {};
	int deg[n] {};
	for (int i = 0; i < n; i++) {
		deg[i] = sz(graph[i]);
	}
	min_pq<pair<int, int>> pq;
	for (int i = 0; i < n; i++) {
		pq.emplace(deg[i], i);
	}
	int ans = 0;
	for (int _ = 0; _ < n; _++) {
		int u;
		while (true) {
			auto [d, v] = pq.top();
			pq.pop();
			if (!del[v] && deg[v] == d) {
				u = v;
				break;
			}
		}
		dbg(u);
		del[u] = true;
		for (auto& a : graph[u]) {
			deg[a]--; 
			pq.emplace(deg[a], a);
		}
		vector<int> cur = {u};
		for (auto& a : graph[u]) {
			if (!del[a]) {
				dbg(a);
				cur.push_back(a);
			}
		}
		int m = sz(cur);
		vector<vector<bool>> cg(m, vector<bool> (m));
		for (int i = 0; i < m; i++) {
			for (int j = i + 1; j < m; j++) {
				if (binary_search(begin(graph[cur[i]]), end(graph[cur[i]]), cur[j])) {
					cg[i][j] = cg[j][i] = true;
				}
			}
		}
		ans = max(ans, clique(cg));
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}