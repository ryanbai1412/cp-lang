//region template
#include "bits/extc++.h"

using namespace std;

template<class T>
void dbgh(T t) {
	cerr << t << endl;
}

template<class T, class... U>
void dbgh(T t, U... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

#define sz(x) static_cast<int>((x).size())
#define long int64_t
#define endl "\n"
//endregion

const int maxn = 2e5;

long ans, asum, arr[maxn], siz[maxn], paths[maxn];
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
	sort(begin(sums), end(sums));
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

void pdfs(int u, int p = -1) {
	siz[u] = arr[u];
	for(auto &v: graph[u]) {
		if(v != p) {
			pdfs(v, u);
			siz[u] += siz[v];
			paths[u] += arr[u] * siz[v];
		}
	}
}

void dfs(int u, int p = -1, long ppaths = 0, long psize = 0) {
	vector<long> cur;
	long cpaths = 0;
	cur.push_back(psize);
	cpaths += ppaths;
	for(auto &v: graph[u]) {
		if(v != p) {
			cpaths += paths[v];
			cur.push_back(siz[v]);
		}
	}
	dbg(solve(cur), asum, arr[u], arr[u] * (asum - arr[u]), cpaths);
	ans = max(ans, solve(cur) + arr[u] * (asum - arr[u]) + cpaths);
	for(auto &v: graph[u]) {
		if(v != p) {
			long csize = asum - siz[v];
			dfs(v, u, (csize - arr[u]) * arr[u], csize);
		}
	}
}

void solve() {
	int n;
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
		asum += arr[i];
	}
	for(int i = 1; i < n; i++) {
		int u;
		cin >> u;
		u--;
		graph[u].push_back(i);
	}
	long add = 0;
	for(int i = 0; i < n; i++) {
		add += arr[i] * (arr[i] - 1);
	}
	pdfs(0);
	dfs(0);
	cout << add + ans << endl;
}

int32_t main() {
	cin.exceptions(ios::failbit);
	cin.tie(0)->sync_with_stdio(0);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	solve();
}