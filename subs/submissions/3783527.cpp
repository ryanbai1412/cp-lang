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
	int n, k, l;
	cin >> n >> k >> l;
	long a[n], b[n];
	for(int i = 0; i < n; i++) {
		cin >> a[i];
	}
	for(int i = 0; i < n; i++) {
		cin >> b[i];
	}
	long dp[n + 1][k + 2][k + 2][l + 2];
	memset(dp, 0xF0, sizeof(dp));
	dp[n][k][k][l] = 0;
	for(int i = n - 1; i >= 0; i--) {
		for(int acnt = 0; acnt <= k; acnt++) {
			for(int bcnt = 0; bcnt <= k; bcnt++) {
				for(int icnt = 0; icnt <= l; icnt++) {
					dp[i][acnt][bcnt][icnt] = max({
						dp[i + 1][acnt][bcnt][icnt],
						a[i] + dp[i + 1][acnt + 1][bcnt][icnt],
						b[i] + dp[i + 1][acnt][bcnt + 1][icnt],
						a[i] + b[i] + dp[i + 1][acnt + 1][bcnt + 1][icnt + 1],
						a[i] + b[i] + dp[i + 1][acnt + 1][bcnt + 1][icnt]
					});
				}
			}
		}
	}
	cout << dp[0][0][0][0] << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}