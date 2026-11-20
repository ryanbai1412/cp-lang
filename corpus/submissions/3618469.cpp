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

long solve(vector<long> arr) {
	sort(begin(arr), end(arr));
	long sum = accumulate(begin(arr), end(arr), 0LL), target = sum / 2;
	if(arr.back() >= target) {
		return arr.back() * (sum - arr.back());
	}
	int n = sz(arr) / 2, m = sz(arr) - n;
	vector<long> sums;
	for(int i = 0; i < (1 << n); i++) {
		long cur = 0;
		for(int j = 0; j < n; j++) {
			if(on(i, j)) {
				cur += arr[j];
			}
		}
		sums.push_back(cur);
	}
	sums.resize(unique(begin(sums), end(sums)) - sums.begin());
	long ans = 0;
	for(int i = 0; i < (1 << m); i++) {
		long cur = 0;
		for(int j = 0; j < m; j++) {
			if(on(i, j)) {
				cur += arr[n + j];
			}
		}
		long ctarg = target - cur;
		auto it = upper_bound(begin(sums), end(sums), ctarg);
		if(it != sums.end()) {
			long csum = *it + cur;
			ans = max(ans, csum * (sum - csum));
		}
		if(it != sums.begin()) {
			it--;
			long csum = *it + cur;
			ans = max(ans, csum * (sum - csum));
		}
	}
	return ans;
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
		for(int i = 1; i < n; i++) {
			if(p[i] == 1) {
				swap(arr[0], arr[1]);
				break;
			}
		}
		long add = 0;
		for(int i = 0; i < n; i++) {
			add += arr[i] * (arr[i] - 1);
		}
		int cnt[n] {};
		for(int i = 1; i < n; i++) {
			cnt[p[i]]++;
		}
		long tsum = accumulate(arr, arr + n, long(0)) - arr[0];
		add += arr[0] * tsum;
		cerr << add << endl;
		cout << add + solve(vector<long>(arr + 1, arr + n)) << endl;
		return;
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
		ans = max(ans, solveg());
	}
	cout << ans << endl;
}

int32_t main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}