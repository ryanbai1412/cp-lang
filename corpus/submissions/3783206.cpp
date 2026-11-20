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

const int maxn = 1e5;

int n, m, ax, bx, cx;
map<int, long> memo;
vector<tuple<int, int, int>> graph[maxn];

long get(long x) {
	return ax * x * x + bx * x + cx;
}

long dp(int u, int t) {
	int key = t * n + u;
	long &ans = memo[key];
	if(!ans) {
		if(u == n - 1) {
			ans = t;
		}else {
			ans = 4e18;
			for(auto &[v, s, e]: graph[u]) {
				if(t <= s) {
					ans = min(ans, get(s - t) + dp(v, e));
					dbg(u, t, v, ans, get(s - t), dp(v, e));
				}
			}
		}
		ans++;
	}
	return ans - 1;
}

void solve() {
	cin >> n >> m >> ax >> bx >> cx;
	for(int i = 0; i < m; i++) {
		int u, v, s, e;
		cin >> u >> v >> s >> e;
		u--;
		v--;
		graph[u].emplace_back(v, s, e);
	}
	cout << dp(0, 0) << endl;
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