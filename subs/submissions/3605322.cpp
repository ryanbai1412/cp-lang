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

void solve() {
	int n, q;
	cin >> n >> q;
	int ans = 0;
	int cnt[1 << (n * 2)] {};
	while(q--) {
		int arr[n][n];
		for(int i = 0; i < n; i++) {
			string s;
			cin >> s;
			for(int j = 0; j < n; j++) {
				arr[i][j] = s[j] == 'R';
			}
		}
		for(int i = 0; i < n - 1; i++) {
			for(int j = 0; j < n - 1; j++) {
				for(auto &x: {1, 0}) {
					for(auto &y: {1, 0}) {
						arr[i + x][j + y] ^= arr[i][j];
					}
				}
			}
		}
		int mask = 0;
		for(int i = 0; i < n; i++) {
			mask = (mask << 1) | arr[i][n - 1];
		}
		for(int i = 0; i < n; i++) {
			mask = (mask << 1) | arr[n - 1][i];
		}
		ans += cnt[mask];
		cnt[mask]++;
	}
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