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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n, ql, qr;
	cin >> n >> ql >> qr;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int ans = 0;
	for (int i = 0; i < n; i++) {
		int mn = INT_MAX, mx = 0;
		for (int j = i; j < n; j++) {
			mn = min(mn, arr[j]);
			mx = max(mx, arr[j]);
			int c = mx - mn;
			if (ql <= c && c <= qr) {
				ans++;
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}