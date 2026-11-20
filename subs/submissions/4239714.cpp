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
	long arr[n - 1];
	for (int i = 0; i < n - 1; i++) {
		cout << "? " << i + 1 << " " << i + 2 << endl;
	}
	cout.flush();
	for (int i = 0; i < n - 1; i++) {
		cin >> arr[i];
		assert(arr[i] != -1);
	}
	assert(n <= 3);
	int vis[n] {}, carr[n];
	int ind = int(min_element(arr, arr + n - 1) - arr);
	for (int i = 1; i <= n; i++) {
		if (arr[ind] % i == 0) {
			long cur = i;
			vis[cur - 1] = true;
			carr[ind] = i;
			for (int j = ind; j < n - 1; j++) {
				if (arr[j] % cur) {
					goto loop;
				}
				cur = arr[j] / cur;
				if (1 <= cur && cur <= n && vis[cur - 1] != i) {
					vis[cur - 1] = i;
				} else {
					goto loop;
				}
				carr[j + 1] = int(cur);
			}
			cur = i;
			for (int j = ind - 1; j >= 0; j--) {
				if (arr[j] % cur) {
					goto loop;
				}
				cur = arr[j] / cur;
				if (1 <= cur && cur <= n && vis[cur - 1] != i) {
					vis[cur - 1] = i;
				} else {
					goto loop;
				}
				carr[j] = int(cur);
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
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}