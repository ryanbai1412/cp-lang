#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr if (false) cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const double phih = (sqrt(5) - 1) / 2, phil = 1 - phih;

int cm1(int l, int r) {
	return int(phih * l + phil * r);
}

int cm2(int l, int r) {
	return int(phil * l + phih * r);
}

template <typename T>
int ternary(int n, const T &t) {
	int l = 1, r = n, m1 = cm1(l, r), m2 = cm2(l, r);
	while (m1 < m2) {
		if (m1 > m2) {
			m1 = cm1(l, r);
			m2 = cm2(l, r);
		}
		assert(m1 <= m2);
		long x1 = t(m1), x2 = t(m2);
		if (x1 > x2) {
			l = m1;
			m1 = m2;
			m2 = cm2(l, r);
		} else if (x1 < x2) {
			r = m2;
			m2 = m1;
			m1 = cm1(l, r);
		} else {
			l = m1;
			r = m2;
			m1 = cm1(l, r);
			m2 = cm2(l, r);
		}
		dbg(l, r, m1, m2);
	}
	pair<long, int> ans {1e18, -1};
	for (int i = l; i <= r; i++) {
		ans = min(ans, pair<long, int> {t(i), i});
	}
	return ans.second;
}

int queries = 0;
long query(int x, int y) {
	static map<pair<int, int>, long> cache;
	auto [it, inserted] = cache.insert({{x, y}, 0});
	if (inserted) {
		assert(queries <= 170);
		queries++;
		cout << "? " << x << " " << y << endl;
		cin >> it->second;
	}
	return it->second;
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	int x = ternary(n, [&](int i) -> long {
		return query(i, 1);
	});
	int y = ternary(m, [&](int i) -> long {
		return query(x, i);
	});
	long ans = query(x, y);
	dbg(queries);
	cout << "! " << ans << endl;
}

int main() {
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}