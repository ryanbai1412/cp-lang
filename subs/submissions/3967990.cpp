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

const int maxn = 3e3 + 5, maxm = 17;
const long mod = 998244352;

int siz[maxn];
long fact[maxn], memo[maxn][maxn][2];
vector<int> graph[maxn];

void pcomp() {
	fact[0] = 1;
	for (int i = 1; i < maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
	}
}

void pdfs(int u) {
	siz[u] = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
		siz[u] += siz[v];
	}
}

long dp(int u, int k, int cur) {
	auto &ans = memo[u][k][cur];
	if (ans != -1) {
		return ans;
	}

	vector<vector<long>> cxdp(maxm, vector<long>(k + 1));
	cxdp[cur][cur] = 1;
	for (auto& v : graph[u]) {
		vector<vector<long>> nxdp(maxm, vector<long>(k + 1));
		for (int a = 0; a < maxm; a++) {
			auto &ndp = nxdp[a];
			for (int i = 0; i <= k; i++) {
				for (int j = 0; i + j <= k; j++) {
					ndp[i + j] = (ndp[i + j] + cxdp[a][i] * dp(v, j, 0)) % mod; 
					if (a && j) {
						ndp[i + j] = (ndp[i + j] + cxdp[a - 1][i] * dp(v, j, 1)) % mod;
					}
				}
			}
		}
		cxdp = nxdp;
	}

	ans = 0;
	for (int i = 0; i < maxm; i++) {
		ans = (ans + cxdp[i][k] * fact[i]) % mod;
	}
	return ans;
}

void solve() {
	int k, n;
	cin >> n >> k;
	if (n <= 20) {
		int edges[n - 1][2];
		for (auto& [u, v] : edges) {
			cin >> u >> v;
			u--;
			v--;
		}
		long ans = 0;
		int perm[n - 1] {};
		fill(perm, perm + k, 1);
		sort(perm, perm + n - 1);
		do {
			int deg[n]{};
			for (int j = 0; j < n - 1; j++) {
				if (perm[j]) {
					auto& [u, v] = edges[j];
					deg[u]++;
					deg[v]++;
				}
			}
			long cur = 1;
			for (int j = 0; j < n; j++) {
				cur = (cur * fact[deg[j]]) % mod;
			}
			ans = (ans + cur) % mod;
		} while(next_permutation(perm, perm + n - 1));
		cout << ans << endl;
		return;
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	memset(memo, -1, sizeof(memo));
	pdfs(0);
	cout << dp(0, k, 0) << endl;
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