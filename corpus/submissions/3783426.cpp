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

const int maxn = 300;
const long mod = 1e9 + 7;

int n, arr[maxn][2];
map<int, long> memo;

long dp(int l, int r, int m) {
	if(l > r) {
		return 1;
	}
	int key = m * n * n + l * n + r;
	long &ans = memo[key];
	if(ans) {
		return ans - 1;
	}
	ans = 0;
	int il = (l + r) / 2 - 2, ir = il + 4;
	for(int i = il; i <= ir; i++) {
		if(l <= i && i <= r && abs((i - l) - (r - i)) <= 2) {
			for(int k = arr[i][0]; k <= min(m, arr[i][1]); k++) {
				ans = (ans + dp(l, i - 1, k) * dp(i + 1, r, k - 1)) % mod;
			}
		}
	}
	ans++;
	return ans - 1;
}

void solve() {
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	cout << dp(0, n - 1, 10000) << endl;
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