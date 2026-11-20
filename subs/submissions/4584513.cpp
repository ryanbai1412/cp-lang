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

struct Fract {
	ld num, den;

	Fract(ld a, ld b) {
		if (b < 0) {
			a = -a;
			b = -b;
		}
		num = a;
		den = b;
	}

	bool operator<(const Fract& f) const {
		return num * f.den < den * f.num;
	}

	bool operator>(const Fract& f) const {
		return num * f.den > den * f.num;
	}

	bool operator<=(const Fract& f) const {
		return num * f.den <= den * f.num;
	}

	bool operator>=(const Fract& f) const {
		return num * f.den >= den * f.num;
	}

	ld eval() const {
		return num / den;
	}
};

void solve() {
	int n;
	cin >> n;
	ld arr[n][2];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	for (auto& [x1, y1] : arr) {
		Fract mn {0, 1}, mx {1000, 1};
		for (auto& [x2, y2] : arr) {
			if (x1 == x2) {
				if (y2 < y1) {
					mn = {int(1e9), 1};
					break;
				}
				continue;
			}
			Fract cur {y1 * y1 - y2 * y2 + x1 * x1 - x2 * x2, 2 * (x1 - x2)};
			if (x1 < x2) {
				dbg(cur <= mx, cur.eval() <= mx.eval(), cur.num, cur.den);
				if (cur <= mx) {
					mx = cur;
				}
			} else {
				if (cur >= mn) {
					dbg(cur.eval() >= mn.eval());
					mn = cur;
				}
			}
		}
		if (mn <= mx) {
			cout << "The sheep at (" << x1 << ", " << y1 << ") might be eaten." << endl;
		}
	}
}

int main() {
	cout << fixed << setprecision(2);
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}