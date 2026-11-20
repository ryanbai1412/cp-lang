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

const int maxn = 1e5 + 5;

vector<int> graph[maxn];
long mod, memo[maxn], cans[maxn];

long dp(int u) {
	long &ans = memo[u];
	if (ans != -1) {
		return ans;
	}
	ans = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		ans = (ans * (1 + dp(v))) % mod;
	}
	return ans;
}

void reroot(int u, long p) {
	cans[u] = (dp(u) * (p + 1)) % mod;
	int n = sz(graph[u]);
	if (!n) {
		return;
	}
	long psum[n], ssum[n];
	psum[0] = 1;
	for (int i = 0; i < n - 1; i++) {
		psum[i + 1] = (psum[i] * (1 + dp(graph[u][i]))) % mod;
	}
	ssum[n - 1] = 1;
	for (int i = n - 1; i >= 1; i--) {
		ssum[i - 1] = (ssum[i] * (1 + dp(graph[u][i]))) % mod;
	}
	for (int i = 0; i < n; i++) {
		long cur = (psum[i] * ssum[i]) % mod;
		dbg(u, i, cur);
		cur = (cur * (p + 1)) % mod;
		reroot(graph[u][i], cur);
	}
}

void solve() {
	int n;
	cin >> n >> mod;
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	memset(memo, -1, sizeof(memo));
	dp(0);
	reroot(0, 0);
	for (int i = 0; i < n; i++) {
		cout << cans[i] << endl;
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