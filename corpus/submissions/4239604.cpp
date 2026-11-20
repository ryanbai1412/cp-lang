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
	int arr[n - 1];
	for (int i = 0; i < n - 1; i++) {
		cout << "? " << i + 1 << " " << i + 2 << endl << flush;
		cin >> arr[i];
		assert(arr[i] != -1);
	}
	for (int i = 1; i <= n; i++) {
		if (arr[0] % i == 0) {
			bool vis[n] {};
			int cur = i;
			int carr[n];
			carr[0] = cur;
			for (int j = 0; j < n - 1; j++) {
				if (arr[j] % cur) {
					goto loop;
				}
				cur = arr[j] / cur;
				carr[j + 1] = cur;
			}
			for (auto& a : carr) {
				if (1 <= a && a <= n && !vis[a - 1]) {
					vis[a - 1] = true;
				} else {
					goto loop;
				}
			}
			cout << "!";
			for (auto& a : carr) {
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