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

const int maxn = 3e3 + 5, maxm = 26;
const long mod = 998244352;

int siz[maxn];
long fact[maxn], memo[maxn][2][maxn];
vector<int> graph[maxn];

void pcomp() {
	fact[0] = 1;
	for (int i = 1; i < maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
	}
	assert((fact[maxm - 1] * maxm) % mod == 0);
}

void pdfs(int u) {
	siz[u] = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
		siz[u] += siz[v];
	}
}

void dp(int u, int cur) {
	if (memo[u][cur][0] != -1) {
		return;
	}

	for (auto& v : graph[u]) {
		dp(v, 0);
		dp(v, 1);
	}

	int acx = 0;
	int k = siz[u];
	long xxdp[2][maxm][k + 1];
	memset(xxdp, 0, sizeof(xxdp));
	int csum = cur;
	xxdp[acx][cur][cur] = 1;
	for (auto& v : graph[u]) {
		auto& cxdp = xxdp[acx];
		auto& nxdp = xxdp[acx ^ 1];
		memset(nxdp, 0, sizeof(nxdp));
		for (int a = 0; a < maxm; a++) {
			auto& ndp = nxdp[a];
			for (int i = 0; i <= csum; i++) {
				for (int j = 0; i + j <= k && j <= siz[v]; j++) {
					ndp[i + j] =
						(ndp[i + j] + cxdp[a][i] * memo[v][0][j]) % mod;
					if (a && j) {
						ndp[i + j] =
							(ndp[i + j] + cxdp[a - 1][i] * memo[v][1][j]) % mod;
					}
				}
			}
		}
		csum += siz[v];
		acx ^= 1;
	}

	auto& cxdp = xxdp[acx];
	for (int i = 0; i <= k; i++) {
		long ans = 0;
		for (int j = 0; j < maxm; j++) {
			ans = (ans + cxdp[j][i] * fact[j]) % mod;
		}
		memo[u][cur][i] = ans;
	}
}

void solve() {
	int k, n;
	cin >> n >> k;
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	memset(memo, -1, sizeof(memo));
	//	int root = chrono::steady_clock::now().time_since_epoch().count() % n;
	int root = 0;
	dbg(root);
	pdfs(root);
	dp(root, 0);
	cout << memo[root][0][k] << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}