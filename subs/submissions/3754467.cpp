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

const int mod = 1e4;

void solve() {
	int n;
	cin >> n;
	int p[n], e[n];
	for(int i = 0; i < n; i++) {
		cin >> p[i] >> e[i];
	}
	int dp[n + 1][2];
	dp[n][0] = 1;
	dp[n][1] = 0;
	for(int i = n - 1; i >= 0; i--) {
		if(p[i] > 2) {
			dp[i][0] = (dp[i + 1][0] + dp[i + 1][1] * (p[i] - 1)) % mod;
			dp[i][1] = (dp[i + 1][1] + dp[i + 1][0] * (p[i] - 1)) % mod;
		}else {
			dp[i][0] = dp[i + 1][0];
			dp[i][1] = dp[i + 1][1];
		}
	}
	cout << dp[0][0] - 1 << endl;
	cout << dp[0][1] << endl;
	cout << 0 << endl;
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