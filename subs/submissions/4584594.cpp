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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct P {
	ld ang;
	int x, y, w;

	P(int _x, int _y, int w) : ang(atan2(_y, _x)), x(_x), y(_y), w(w) {
		int g = gcd(x, y);
		x /= g;
		y /= g;
	}

	bool operator<(const P& p) const {
		return ang < p.ang;
	}

	bool opposite(const P& p) const {
		return x == -p.x && y == -p.y;
	}

	bool same(const P& p) const {
		return x == p.x && y == p.y;
	}
};

ld deg(int x, int y) {
	int g = gcd(x, y);
	x /= g;
	y /= g;
	return atan2(ld(y), ld(x));
}

void solve() {
	int n;
	cin >> n;
	vector<array<int, 3>> in(n);
	for (auto& [x, y, w] : in) {
		cin >> x >> y >> w;
	}
	long ans = 0;
	for (auto& [x, y, w] : in) {
		vector<P> arr;
		for (auto& [x1, y1, w1] : in) {
			if (tie(x, y) != tie(x1, y1)) {
				arr.emplace_back(x1 - x, y1 - y, w1);
			}
		}
		sort(begin(arr), end(arr));
		auto tmp = arr;
		arr.insert(arr.end(), begin(tmp), end(tmp));
		int r = 0;
		long cur = 0;
		for (int l = 0; l < n - 1;) {
			for (; r < sz(arr) && arr[l].opposite(arr[r]); r++) {
				cur += arr[r].w;
			}
			ans = max(ans, cur + w);
			P init = arr[l];
			for (; l < n - 1 && arr[l].same(init); l++) {
				cur -= arr[l].w;
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}