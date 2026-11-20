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

bool vis[maxn][2];
int siz[maxn], memo[maxn][2][maxn];
vector<pair<int, int>> graph[maxn];

void pdfs(int u) {
	siz[u] = 1;
	for(auto &[v, w]: graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), pair<int, int> {u, w}));
		pdfs(v);
		siz[u] += siz[v];
	}
}

template<bool add, bool cur>
void dp(int u) {
	static int ndp[maxn];
	if(vis[u][cur]) {
		return;
	}
	vis[u][cur] = true;
	auto cdp = memo[u][cur];
	fill(cdp + 1, cdp + maxn, 1e9);
	int csize = 1;
	for(auto &[v, w]: graph[u]) {
		dp<add, 0>(v);
		dp<add, 1>(v);
		fill(ndp, ndp + maxn, 1e9);
		for(int i = 0; i <= csize; i++) {
			for(int j = 0; j <= siz[v] && i + j <= siz[u]; j++) {
				if(j < siz[v]) {
					ndp[i + j] = min(ndp[i + j], (add && !cur) * w + cdp[i] + memo[v][0][j]);
				}
				if(j) {
					ndp[i + j] = min(ndp[i + j], cur * w + cdp[i] + memo[v][1][j - 1]);
				}
			}
		}
		memcpy(cdp, ndp, sizeof(ndp));
		csize += siz[v];
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
	pdfs(0);
	if(m == 2) {
		dp<true, 1>(0);
	}else {
		dp<false, 1>(0);
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