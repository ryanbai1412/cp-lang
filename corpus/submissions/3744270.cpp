#pragma GCC optimize("Ofast,unroll-loops")

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

const int maxn = 105, inf = 0x3f3f3f3f;

long opt[maxn][maxn], dp[maxn][maxn][maxn];

inline int sq(int x) {
	return x * x;
}

void solve() {
	double da, db;
	int n, m;
	cin >> da >> db >> n >> m;
	n--;
	m--;
	int a = round(da * 10), b = round(db * 10);
	for(auto &aa: opt) {
		for(auto &bb: aa) {
			bb = 1e9;
		}
	}
	for(int i = 0; i < maxn; i++) {
		opt[i][0] = i * i;
	}
	for(int i = 1; i < maxn; i++) {
		for(int j = 1; j < maxn; j++) {
			for(int k = 0; k < i; k++) {
				opt[i][j] = min(opt[i][j], opt[k][j - 1] + sq(i - k));
			}
		}
	}
	for(auto &aa: opt) {
		for(auto &bb: aa) {
			bb *= b;
		}
	}
	for(auto &aa: dp) {
		for(auto &bb: aa) {
			fill(begin(bb), end(bb), 1e9);
		}
	}
	dp[n][m][100] = 0;
	for(int j = 0; j <= m; j++) {
		for(int k = 0; k < 100; k++) {
			if(opt[100 - k][m - j] < inf) {
				dp[k][n][j] = a * sq(100 - k) + opt[100 - k][m - j];
			}
		}
	}
	for(int k = 99; k >= 0; k--) {
		for(int i = n - 1; i >= 0; i--) {
			for(int j = m - 1; j >= 0; j--) {
				for(int nk = k + 1; nk <= 100 - (n - i); nk++) {
					const int cx = a * sq(nk - k);
#pragma GCC unroll maxn
					for(int bc = 0; bc < min(nk - k, m - j); bc++) {
						dp[k][i][j] = min(dp[k][i][j], cx + opt[nk - k][bc] + dp[nk][i + 1][j + bc + 1]);
					}
				}
			}
		}
	}
	cout << dp[0][0][0] / 10.0 << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	cout << fixed << setprecision(1);
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}