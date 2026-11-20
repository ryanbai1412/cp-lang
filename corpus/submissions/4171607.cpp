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

template <typename T, int min, int max>
struct AR {
	T arr[max - min + 1] {};

	T& operator[](int ind) {
		return arr[ind - min];
	}

	T& operator[](int ind) const {
		return arr[ind - min];
	}
};

const int maxn = 1e3 + 5;

AR<int, -maxn, maxn> dpx, dpy;

void solve() {
	int n;
	cin >> n;
	int xs[n + 1], ys[n + 1];
	xs[0] = ys[0] = 0;
	for (int i = 1; i <= n; i++) {
		cin >> xs[i] >> ys[i];
	}
	auto distx = [&](int i, int j) -> int { return abs(xs[i] - xs[j]); };
	auto disty = [&](int i, int j) -> int { return abs(ys[i] - ys[j]); };
	for (int ci = n - 1; ci >= 0; ci--) {
		int dx = distx(ci, ci + 1), dy = disty(ci, ci + 1);
		int ax = dpx[ys[ci]], ay = dpy[xs[ci]];
		{
			int ca = dx, cb = ys[ci + 1] + ay;
			bool prev = false;
			for (int i = -maxn; i < ys[ci + 1]; i++) {
				bool cx = cb - i <= ca + dpx[i];
				assert(prev <= cx);
				prev = cx;
				dpx[i] = min(ca + dpx[i], cb - i);
			}
		}
		{
			int ca = dx, cb = -ys[ci + 1] + ay;
			bool prev = false;
			for (int i = ys[ci + 1]; i <= maxn; i++) {
				bool cx = ca + dpx[i] <= cb + i;
				assert(prev <= cx);
				prev = cx;
				dpx[i] = min(ca + dpx[i], cb + i);
			}
		}
		{
			int ca = dy, cb = xs[ci + 1] + ax;
			bool prev = false;
			for (int i = -maxn; i < xs[ci + 1]; i++) {
				bool cx = cb - i <= ca + dpy[i];
				assert(prev <= cx);
				prev = cx;
				dpy[i] = min(ca + dpy[i], cb - i);
			}
		}
		{
			int ca = dy, cb = -xs[ci + 1] + ax;
			bool prev = false;
			for (int i = xs[ci + 1]; i <= maxn; i++) {
				bool cx = ca + dpy[i] <= cb + i;
				assert(prev <= cx);
				prev = cx;
				dpy[i] = min(ca + dpy[i], cb + i);
			}
		}
	}
	cout << dpx[0] << endl;
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