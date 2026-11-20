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

const long mod = 65521;

const int maxn = 100;

long ans = 0;
int n, arr[maxn][maxn];
bool vis[maxn];

void dfs(int ind = 0, long cur = 1, int inv = 0) {
	if(!cur) {
		return;
	}
	if(ind == n) {
		if(inv & 1) {
			ans = (ans + mod - cur) % mod;
		}else {
			ans = (ans + cur) % mod;
		}
		return;
	}
	int cnt = 0;
	for(int i = n - 1; i >= 0; i--) {
		if(!vis[i]) {
			vis[i] = true;
			dfs(ind + 1, (cur * arr[ind][i]) % mod, inv + cnt);
			vis[i] = false;
		}else {
			cnt++;
		}
	}
}

void solve() {
	int k;
	cin >> k >> n;
	int deg[n] {};
	bool graph[n][n] {};
	for(int i = 0; i < n; i++) {
		for(int j = i + 1; j <= i + k && j < n; j++) {
			deg[i]++;
			deg[j]++;
			graph[i][j] = graph[j][i] = true;
		}
	}
	n--;
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < n; j++) {
			if(i == j) {
				arr[i][i] = deg[i];
			}else if(graph[i][j]) {
				arr[i][j] = -1;
			}else {
				arr[i][j] = 0;
			}
		}
	}
	dfs();
	cout << ans << endl;
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