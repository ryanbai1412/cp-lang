#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, m;
	cin >> n >> m;
	vector<int> graph[n];
	for(int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	int ans = 0;
	vector<int> oans;
	for(int i = 0; i < 1 << n; i++) {
		int tot = 0;
		vector<int> cur(n);
		for(int j = 0; j < n; j++) {
			if((i >> j) & 1) {
				cur[j] = 1;
			}
		}
		for(int j = 0; j < n; j++) {
			int tmp = 0;
			for(auto &a: graph[j]) {
				if(cur[a] == cur[j]) {
					tmp++;
				}
			}
			if((tmp & 1) == 0) {
				tot++;
			}
		}
		if(tot > ans) {
			ans = tot;
			oans = cur;
		}
	}
	cout << ans << endl;
	for(int i = 0; i < n; i++) {
		if(i) {
			cout << " ";
		}
		cout << oans[i] + 1;
	}
	cout << endl;
}