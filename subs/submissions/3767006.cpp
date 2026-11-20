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

const long mod = 65521, infmod = long(4e18) / mod * mod;

struct mask {
	int x;

	bool operator[](int i) {
		return !!((x >> i) & 1);
	}
};

void solve() {
	int k, n;
	cin >> k >> n;
	n--;
	auto get = [&](int u, int v) -> int {
		if(!(0 <= u && u < n && 0 <= v && v < n)) {
			return 0;
		}
		if(u == v) {
			return min(k, u) + min(k, n - u);
		}else if(abs(u - v) <= k) {
			return -1;
		}
		return 0;
	};
	auto next = [&](int j, int ind) -> int {
		return (j | (1 << ind)) >> 1;
	};
	auto conv = [&](int ind, int rel) -> int {
		return ind + rel - k;
	};
	int m = (1 << (2 * k));
	long dp[n + 1][m];
	memset(dp[n], 0, sizeof(dp[n]));
	dp[n][(1 << k) - 1] = 1;
	for(int i = n - 1; i >= 0; i--) {
		for(int ji = 0; ji < m; ji++) {
			mask j {ji};
			if(!j[0]) {
				int psum = 0;
				for(int a = 2 * k; a >= 1; a--) {
					psum += j[a];
				}
				int mul = 1;
				if(psum & 1) {
					mul = -1;
				}
				dp[i][ji] = mul * get(i, conv(i, 0)) * dp[i + 1][next(ji, 0)];
			}else {
				dp[i][ji] = 0;
				int psum = 0;
				for(int a = 2 * k; a >= 0; a--) {
					if(!j[a]) {
						int mul = 1;
						if(psum & 1) {
							mul = -1;
						}
						dp[i][ji] += mul * get(i, conv(i, a)) * dp[i + 1][next(ji, a)];
					}else {
						psum++;
					}
				}
				dp[i][ji] += infmod;
				dp[i][ji] %= mod;
			}
		}
	}
	cout << dp[0][(1 << k) - 1] << endl;
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