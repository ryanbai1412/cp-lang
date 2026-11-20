//region template
#include "bits/extc++.h"

using namespace std;

#define sz(x) static_cast<int>((x).size())
#define long int64_t
#define endl "\n"
//endregion

const int maxn = 2e5;

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

long solveg() {

}

void solve() {
	int n;
	cin >> n;
	long arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	int p[n];
	for(int i = 1; i < n; i++) {
		cin >> p[i];
		p[i]--;
	}
	if(n == 37) {
		long add = 0;
		for(int i = 0; i < n; i++) {
			add += (arr[i] * arr[i] - 1);
		}
		int cnt[n] {};
		for(int i = 1; i < n; i++) {
			cnt[p[i]]++;
		}
		add += arr[0] * (accumulate(arr, arr + n, long(0)) - arr[0]);
		cerr << add << endl;

//		return;
	}
	auto solveg = [&]() -> long {
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
		return cur;
	};
#ifdef LOCAL
	for(int i = 1; i <= 18; i++) {
		graph[0].push_back(i);
		graph[i + 18].push_back(0);
	}
	cerr << solveg() << endl;
#endif
	long ans = 0;
	for(int i = 0; i < 0; i++) {
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
		ans = max(ans, solveg());
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