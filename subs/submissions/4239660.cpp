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

void solve() {
	int n;
	cin >> n;
	int arr[n - 1], perm[n];
	iota(perm, perm + n, 0);
	shuffle(perm, perm + n, mt19937(chrono::steady_clock::now().time_since_epoch().count()));
	for (int i = 0; i < n - 1; i++) {
		cout << "? " << perm[i] + 1 << " " << perm[i + 1] + 1 << endl;
		cin >> arr[i];
		assert(arr[i] != -1);
	}
	int vis[n] {}, carr[n], narr[n];
	for (int i = 1; i <= n; i++) {
		if (arr[0] % i == 0) {
			int cur = i;
			vis[cur - 1] = true;
			carr[0] = cur;
			for (int j = 0; j < n - 1; j++) {
				if (arr[j] % cur) {
					goto loop;
				}
				cur = arr[j] / cur;
				if (1 <= cur && cur <= n && vis[cur - 1] != i) {
					vis[cur - 1] = i;
				} else {
					goto loop;
				}
				carr[j + 1] = cur;
			}
			for (int j = 0; j < n; j++) {
				narr[perm[j]] = carr[j];
			}
			cout << "!";
			for (auto& a : narr) {
				cout << " " << a;
			}
			cout << endl;
			return;
		loop:;
		}
	}
	assert(false);
}

int main() {
	ios_base::sync_with_stdio(false);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}