#pragma GCC optimize("Ofast,unroll-loops,no-stack-protector")
#pragma GCC target("sse4.1,sse4.2")

#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 305;

int siz[maxn], memo[maxn][2][maxn];
vector<pair<int, int>> graph[maxn];

template<bool add>
void dp(int u, int p = -1) {
	static int xdp[2][2][maxn];
	siz[u] = 1;
	for(auto &[v, _]: graph[u]) {
		if(v == p) {
			continue;
		}
		dp<add>(v, u);
		siz[u] += siz[v];
	}
	int prev = 1, cur = 0;
	for(int i = 0; i < 2; i++) {
		memset(xdp[cur][i], 0x3f, sizeof(int) * (siz[u] + 1));
	}
	xdp[cur][0][0] = xdp[cur][1][0] = 0;
	int csize = 1;
	for(auto &[v, w]: graph[u]) {
		if(v == p) {
			continue;
		}
		auto mv0 = memo[v][0], mv1 = memo[v][1];
		swap(prev, cur);
		auto ndp = xdp[cur];
		auto cdp = xdp[prev];
		for(int i = 0; i < 2; i++) {
			memset(xdp[cur][i], 0x3f, sizeof(int) * (siz[u] + 1));
		}
		for(int i = 0; i <= csize; i++) {
			{
				int end = min(siz[v] - 1, siz[u] - i);
#pragma GCC unroll 300
#pragma GCC ivdep
				for(int j = 0; j <= end; j++) {
					ndp[0][i + j] = min(ndp[0][i + j], add * w + cdp[0][i] + mv0[j]);
					ndp[1][i + j] = min(ndp[1][i + j], cdp[1][i] + mv0[j]);
				}
			}
			{
				int end = min(siz[v], siz[u] - i);
#pragma GCC unroll 300
#pragma GCC ivdep
				for(int j = 1; j <= end; j++) {
					ndp[0][i + j] = min(ndp[0][i + j], cdp[0][i] + mv1[j - 1]);
					ndp[1][i + j] = min(ndp[1][i + j], w + cdp[1][i] + mv1[j - 1]);
				}
			}
		}
		csize += siz[v];
	}
	for(int i = 0; i < 2; i++) {
		memcpy(memo[u][i], xdp[cur][i], sizeof(int) * (siz[u] + 1));
	}
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	if(m - 1 + k > n) {
		cout << -1 << endl;
		return;
	}
	for(int i = 0; i < n - 1; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	if(m == 2) {
		dp<true>(0);
	}else {
		dp<false>(0);
	}
	cout << memo[0][1][k - 1] << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}