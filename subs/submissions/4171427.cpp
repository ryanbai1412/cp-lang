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
	int xs[n + 1], ys[n + 1];
	xs[0] = ys[0] = 0;
	for (int i = 1; i <= n; i++) {
		cin >> xs[i] >> ys[i];
	}
	auto distx = [&](int i, int j) -> int {
		return abs(xs[i] - xs[j]);
	};
	auto disty = [&](int i, int j) -> int {
		return abs(ys[i] - ys[j]);
	};
	int dpx[n + 1][n + 1], dpy[n + 1][n + 1];
	memset(dpx[n], 0, sizeof(dpx[n]));
	memset(dpy[n], 0, sizeof(dpy[n]));
	for (int ci = n - 1; ci >= 0; ci--) {
		for (int pi = ci; pi >= 0; pi--) {
			dpx[ci][pi] = min(distx(ci, ci + 1) + dpx[ci + 1][pi], disty(pi, ci + 1) + dpy[ci + 1][ci]);
			dpy[ci][pi] = min(disty(ci, ci + 1) + dpy[ci + 1][pi], distx(pi, ci + 1) + dpx[ci + 1][ci]);
		}
	}
	cout << dpx[0][0] << endl;
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