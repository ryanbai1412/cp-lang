//region template
#include "bits/extc++.h"

using namespace std;

#define sz(x) static_cast<int>((x).size())
#define long int64_t
#define endl "\n"
//endregion

const int maxn = 10;

bool vis[maxn];
vector<int> graph[maxn];

inline bool on(int mask, int bit) {
	return static_cast<bool>((mask >> bit) & 1);
}

void dfs(int u) {
	vis[u] = true;
	for(auto &v: graph[u]) {
		if(!vis[v]) {
			dfs(v);
		}
	}
}

void solve() {
	int n;
	cin >> n;
	assert(n <= 10);
	long arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	int p[n];
	for(int i = 1; i < n; i++) {
		cin >> p[i];
		p[i]--;
		cerr << p[i] << endl;
	}
	long ans = 0;
	for(int i = 0; i < (1 << n); i++) {
		for(int j = 0; j < n; j++) {
			graph[j].clear();
		}
		for(int j = 1; j < n; j++) {
			if(on(i, j)) {
				graph[p[j]].push_back(j);
			}else {
				graph[j].push_back(p[j]);
			}
		}
		long cur = 0;
		for(int j = 0; j < n; j++) {
			memset(vis, 0, sizeof(vis));
			dfs(j);
			for(int k = 0; k < n; k++) {
				if(vis[k] && k != j) {
					cur += arr[j] * arr[k];
				}
			}
			cur += arr[j] * (arr[j] - 1);
		}
		ans = max(ans, cur);
	}
	cout << ans << endl;
}

int main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}