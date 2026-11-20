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
	int n, m;
	cin >> n >> m;
	double arr[n][3];
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < 3; j++) {
			cin >> arr[i][j];
		}
	}
	double ans = 0;
	for(int i = 0; i < (1 << (2 * n)); i++) {
		double cash = m, a = 0, b = 0;
		int ci = i;
		for(auto &[ca, cb, rate]: arr) {
			int cur = ci & 3;
			ci >>= 2;
			// sell
			if(cur & 1) {
				cash += a * ca;
				cash += b * cb;
				a = b = 0;
			}
			// buy
			if(cur & 2) {
				double unit = ca * rate + cb;
				double ucnt = cash / unit;
				a += rate * ucnt;
				b += ucnt;
				cash = 0;
			}
		}
		ans = max(ans, cash);
	}
	cout << ans << endl;
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