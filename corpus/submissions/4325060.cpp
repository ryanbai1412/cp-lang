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
	string s;
	cin >> s;
	bool h = false, v = false;
	for (auto& a : s) {
		if (a == 'H') {
			h ^= true;
		} else {
			v ^= true;
		}
	}
	int arr[2][2] {{1, 2}, {3, 4}};
	if (h) {
		swap(arr[0][0], arr[0][1]);
		swap(arr[1][0], arr[1][1]);
	}
	if (v) {
		swap(arr[0][0], arr[1][0]);
		swap(arr[0][1], arr[1][1]);
	}
	for (int i = 0; i < 2; i++) {
		for (int j = 0; j < 2; j++) {
			cout << arr[i][j] << " \n"[j == 1];
		}
	}
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