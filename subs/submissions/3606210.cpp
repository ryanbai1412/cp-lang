#include "bits/extc++.h"

using namespace std;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

const long inf = 1e18;

long solve1(int n, int m) {
	if(n == 0 || m == 0) {
		return 0;
	}
	long ans = inf;
	if(n * 2 <= m) {
		ans = min(ans, 1 + solve1(n, m - n * 2));
	}
	if(n % 2 == 0 && n / 2 <= m) {
		ans = min(ans, 1 + solve1(n, m - n / 2));
	}
	if(m * 2 <= n) {
		ans = min(ans, 1 + solve1(m, n - m * 2));
	}
	if(m % 2 == 0 && m / 2 <= n) {
		ans = min(ans, 1 + solve1(m, n - m / 2));
	}
	return ans;
}

long solve2(int n, int m) {
	if(n == 0 || m == 0) {
		return 0;
	}
	long ans = 0;
	if(n * 2 <= m) {
		ans = max(ans, 1 + solve2(n, m - n * 2));
	}
	if(n % 2 == 0 && n / 2 <= m) {
		ans = max(ans, 1 + solve2(n, m - n / 2));
	}
	if(m * 2 <= n) {
		ans = max(ans, 1 + solve2(m, n - m * 2));
	}
	if(m % 2 == 0 && m / 2 <= n) {
		ans = max(ans, 1 + solve2(m, n - m / 2));
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	cout << solve1(n, m) << " " << solve2(n, m) << endl;
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