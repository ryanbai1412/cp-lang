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

bool on(int x, int i) {
	return (x >> i) & 1;
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	vector<int> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	long dp[1 << k][n] {};
	for (int i = 0; i < n; i++) {
		dp[1 << arr[i]][i] = 1;
	}
	for (int i = 0; i < (1 << k); i++) {
		for (int j = 0; j < n; j++) {
			if (on(i, arr[j])) {
				for (auto& v : graph[j]) {
					dp[i][j] += dp[i ^ (1 << arr[j])][v];
				}
			}
		}
	}
	long ans = 0;
	for (int i = 0; i < (1 << k); i++) {
		if (__builtin_popcount(i) >= 2) {
			for (int j = 0; j < n; j++) {
				ans += dp[i][j];
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}