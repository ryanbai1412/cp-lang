#include "bits/stdc++.h"

using namespace std;

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())


struct DSU {
	vector<int> par, size;
	vector<vector<int>> comps;

	DSU(int n): par(n), size(n, 1), comps(n) {
		iota(begin(par), end(par), 0);
		for(int i = 0; i < n; i++) {
			comps[i] = {i};
		}
	}

	int find(int u) {
		return par[u] == u ? u : (par[u] = find(par[u]));
	}

	void merge(int u, int v) {
		u = find(u);
		v = find(v);
		if(size[u] > size[v]) {
			swap(u, v);
		}
		par[u] = par[v];
		comps[v].insert(comps[v].end(), begin(comps[u]), end(comps[u]));
		comps[u].clear();
	}
};

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, m;
	cin >> n >> m;
	DSU dsu(n);
	long arr[n];
	map<long, int> vals;
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
		vals[arr[i]] = i;
	}
	for(int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		dsu.merge(u, v);
	}
	set<pair<int, int>> comps;
	for(int i = 0; i < n; i++) {
		int u = dsu.find(i), size = dsu.size[u];
		comps.emplace(size, u);
	}
	long ans = 0;
	while(sz(comps) > 1) {
		int u = comps.begin()->second;
		auto ccomp = dsu.comps[u];
		for(auto &a: ccomp) {
			vals.erase(arr[a]);
		}
		pair<long, int> mn = {1e18, -1};
		for(auto &a: ccomp) {
			auto it = vals.upper_bound(arr[a]);
			if(it != vals.end()) {
				mn = min(mn, make_pair(it->first - arr[a], it->second));
			}
			if(it != vals.begin()) {
				--it;
				mn = min(mn, make_pair(arr[a] - it->first, it->second));
			}
		}
		assert(mn.second != -1);
		int v = dsu.find(mn.second);
		assert(u != v);
		ans += mn.first;
		comps.erase(comps.begin());
		comps.erase(comps.lower_bound({dsu.size[v], v}));
		dsu.merge(u, v);
		int newu = dsu.find(u);
		comps.emplace(dsu.size[newu], newu);
		for(auto &a: ccomp) {
			vals[arr[a]] = a;
		}
	}
	cout << ans << endl;
}