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

void solve() {
	int m;
	cin >> m;
	int n = 1 << m;
	int arr[n], cost[n], flow[n][n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	for(int i = 0; i < n; i++) {
		cin >> cost[i];
	}
	for(int i = 0; i < n; i++) {
		flow[i][i] = 0;
		for(int j = i + 1; j < n; j++) {
			cin >> flow[i][j];
			flow[j][i] = flow[i][j];
		}
	}
	int psum[n][n + 1];
	for(int i = 0; i < n; i++) {
		partial_sum(flow[i], flow[i] + n, psum[i] + 1);
		psum[i][0] = 0;
	}
	long dp[n << 1][n << 1];
	for(int d = m; d >= 0; d--) {
		for(int u = (1 << d); u < (1 << (d + 1)); u++) {
			for(int par = 0; par < (1 << d); par++) {
				for(int cnt = 0; cnt < (1 << (m - d + 1)); cnt++) {
					long &cur = dp[u][(cnt << d) | par];
					if(d == m) {
						cur = 0;
						int cu = u - n;
						if(arr[cu] ^ cnt) {
							cur += cost[cu];
						}
						for(int pd = m - 1; pd >= 0; pd--) {
							if(((par >> pd) & 1) ^ cnt) {
								int ls = m - pd;
								int p = (cu >> ls) << ls;
								if(!((cu >> (ls - 1)) & 1)) {
									p += (1 << (ls - 1));
								}
								cur += psum[cu][p + (1 << (ls - 1))] - psum[cu][p];
							}
						}
					}else {
						cur = 4e18;
						int csz = 1 << (m - d);
						for(int acnt = 0; acnt < csz; acnt++) {
							int bcnt = cnt - acnt;
							if(0 <= bcnt && bcnt < csz) {
								int cflag = ((csz - 1 - acnt < bcnt) << d) | par;
								cur = min(cur, dp[u << 1][(acnt << (d + 1)) | cflag] +
											   dp[(u << 1) | 1][(bcnt << (d + 1)) | cflag]);
							}
						}
					}
				}
			}
		}
	}
	cout << *min_element(dp[1], dp[1] + (n << 1)) << endl;
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