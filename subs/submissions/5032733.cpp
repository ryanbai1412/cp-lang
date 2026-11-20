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

const int maxn = 1000;

int dist[maxn][maxn], nxt[maxn][maxn];
double memo[maxn][maxn];
vector<int> graph[maxn];

double dp(int a, int b) {
	double &ans = memo[a][b];
	if(ans != -1) {
		return ans;
	}else if(dist[a][b] <= 2) {
		return ans = !!dist[a][b];
	}
	int na = nxt[nxt[a][b]][b];
	ans = dp(na, b);
	for(auto &v: graph[b]) {
		ans += dp(na, v);
	}
	ans = 1 + ans / (sz(graph[b]) + 1);
	dbg(a, b, ans);
	return ans;
}

void solve() {
	int n, m, a, b;
	cin >> n >> m >> a >> b;
	a--;
	b--;
	for(int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	memset(dist, -1, sizeof(dist));
	for(int i = 0; i < n; i++) {
		auto dist = ::dist[i];
		dist[i] = 0;
		vector<int> q, nq;
		nq.push_back(i);
		while(sz(nq)) {
			q = nq;
			sort(begin(q), end(q));
			nq.clear();
			for(auto &u: q) {
				for(auto &v: graph[u]) {
					if(dist[v] == -1) {
						dist[v] = dist[u] + 1;
						nxt[v][i] = u;
						nq.push_back(v);
					}
				}
			}
		}
	}
	for(int i = 0; i < n; i++) {
		fill(memo[i], memo[i] + n, -1);
	}
	cout << dp(a, b) << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	cout << fixed << setprecision(3);
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}