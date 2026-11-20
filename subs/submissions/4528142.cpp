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

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int dist[n][n];
	memset(dist, 0x3f, sizeof(dist));
	for (int i = 0; i < n - 1; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		dist[u][v] = dist[v][u] = w;
	}
	for (int i = 0; i < n; i++) {
		dist[i][i] = 0;
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			for (int k = 0; k < n; k++) {
				dist[j][k] = min(dist[j][k], dist[j][i] + dist[i][k]);
			}
		}
	}
	pair<int, int> ans {1e9, 1e9};
	for (int i = 1; i < (1 << n); i++) {
		int ci = __builtin_popcount(i);
		if (ci > m) {
			continue;
		}
		int cdist[n], cans = 0;
		memset(cdist, 0x3f, sizeof(cdist));
		for (int j = 0; j < n; j++) {
			if ((i >> j) & 1) {
				cans += arr[j];
				for (int k = 0; k < n; k++) {
					cdist[k] = min(cdist[k], dist[j][k]);
				}
			}
		}
		ans = min(ans, pair<int, int> {cans + accumulate(cdist, cdist + n, 0), ci});
	}
	cout << ans.first << endl;
	cout << ans.second << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}