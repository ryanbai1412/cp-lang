#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
		 << ": ";                                          \
	dbgh(__VA_ARGS__)
#else
#define cerr   \
	if (false) \
	cerr
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
	int n, k;
	cin >> n >> k;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int ans = n;
	for (int i = 0; i < (1 << n); i++) {
		bool vis[n] {};
		for (int j = 0; j < n; j++) {
			if ((i >> j) & 1) {
				{
					int ind = j, cnt = k;
					while (ind < n) {
						vis[ind] = true;
						if (!cnt || ind + 1 >= n) {
							break;
						}
						if (arr[ind] > arr[ind + 1]) {
							ind++;
							cnt = k;
						} else if (arr[ind] == arr[ind + 1]) {
							ind++;
							cnt--;
						} else {
							break;
						}
					}
				}
				{
					int ind = j, cnt = k;
					while(ind >= 0) {
						vis[ind] = true;
						if (!cnt || ind - 1 < 0) {
							break;
						}
						if (arr[ind] > arr[ind - 1]) {
							ind--;
							cnt = k;
						} else if (arr[ind] == arr[ind - 1]) {
							ind--;
							cnt--;
						} else {
							break;
						}
					}
				}
			}
		}
		for (auto& a : vis) {
			if (!a) {
				goto loop;
			}
		}
		ans = min(ans, __builtin_popcount(i));
		loop:;
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
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}