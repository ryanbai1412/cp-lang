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

const int maxn = 205;
const long mod = 1e9 + 7;

long ans1, ans2;
int k, garr[maxn][2];
vector<int> st, graph[maxn];

void csolve() {
	int n = sz(st);
	int arr[n][2];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < 2; j++) {
			arr[i][j] = garr[st[i]][j];
		}
	}
	int mn = 1e9, mx = 1e9;
	for (auto& [l, r] : arr) {
		mn = min(mn, l);
		mx = min(mx, r);
	}
	long cur = 0;
	for (int i = mn; i <= mx; i++) {
		{
			long dp[2] {1, 0};
			for (auto& [l, r] : arr) {
				long a = dp[0] * max(0, min(r, i + k) - max(l, i + 1) + 1);
				long b = dp[1] * max(0, min(r, i + k) - max(l, i) + 1);
				if (l <= i && i <= r) {
					b += dp[0];
				}
				dp[0] = a % mod;
				dp[1] = b % mod;
			}
			ans1 = (ans1 + dp[1]) % mod;
			cur += dp[1];
		}
	}
	dbg(st[0], st.back(), mn, mx, cur);
}

void dfs(int u, int p = -1) {
	st.push_back(u);
	if (st[0] <= u) {
		csolve();
	}
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	st.pop_back();
}

void solve() {
	int n;
	cin >> n >> k;
	for (int i = 0; i < n; i++) {
		cin >> garr[i][0] >> garr[i][1];
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		dfs(i);
	}
	cout << ans1 << endl;
	cout << ans2 << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}