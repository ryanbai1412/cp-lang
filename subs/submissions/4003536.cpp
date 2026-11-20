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

const int maxn = 5e5, logn = 20;
const long mod = 998244353;

int t, tin[maxn], tout[maxn], lift[logn][maxn];
vector<int> graph[maxn];

long bpow(long base, long exp) {
	long ans = 1;
	while(exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp /= 2;
	}
	return ans;
}

long inv(long x) {
	return bpow(x, mod - 2);
}

long deter(vector<vector<long>> arr) {
	int n = sz(arr);
	long mul = 1;
	for (int _ = 0; _ < n; _++) {
		for (int i = 0; i < n; i++) {
			if (!arr[i][i]) {
				int j = i + 1;
				while (j != i) {
					if (j == n) {
						j = 0;
					}
					if (arr[i][j]) {
						swap(i, j);
						mul *= -1;
						goto loop;
					}
					j++;
				}
				return 0;
			}
		loop:;
			long cinv = inv(arr[i][i]);
			for (int j = i + 1; j < n; j++) {
				long cmul = (arr[j][i] * cinv) % mod;
				for (int k = 0; k < n; k++) {
					arr[j][k] = (arr[j][k] - arr[i][k] * cmul) % mod;
				}
			}
		}
	}
	for (int i = 0; i < n; i++) {
		mul = (mul * arr[i][i]) % mod;
	}
	if (mul < 0) {
		mul += mod;
	}
	return mul;
}

void dfs(int u, int p = -1) {
	tin[u] = t++;
	lift[0][u] = p;
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	tout[u] = t - 1;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int lca(int u, int v) {
	if (anc(u, v)) {
		return u;
	}
	for (int i = logn - 1; i >= 0; i--) {
		if (lift[i][u] != -1 && !anc(lift[i][u], v)) {
			u = lift[i][u];
		}
	}
	return lift[0][u];
}

void solve() {
	int n, k;
	cin >> n >> k;
	if (k > n) {
		cout << 0 << endl;
		return;
	}
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int lcarr[k];
	for (auto& a : lcarr) {
		cin >> a;
		a--;
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	dfs(0);
	vector<vector<long>> mat(k, vector<long>(k));
	for (int i = 0; i < k; i++) {
		for (int j = 0; j < k; j++) {
			mat[i][j] = arr[lca(lcarr[i], lcarr[j])];
		}
	}
	cout << deter(mat) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}