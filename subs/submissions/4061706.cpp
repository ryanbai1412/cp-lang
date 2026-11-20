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

char buf[1 << 21];

void init() {
	fread_unlocked(buf, 1, 1 << 21, stdin);
}

void read_int(int& x) {
	static int ind = 0;
	char c;
	while ((c = buf[ind++]) < '0' || c > '9')
		;
	x = c - '0';
	while ((c = buf[ind++]) >= '0' && c <= '9') {
		x = x * 10 + c - '0';
	}
}

const int maxn = 1.1e7;

template <int n>
struct Fact {
	int mp[n + 1];

	Fact() {
		iota(begin(mp), end(mp), 0);
		for (int i = 2; i <= n; i++) {
			if (mp[i] == i && n / i >= i) {
				for (int j = i * i; j <= n; j += i) {
					if (mp[j] == j) {
						mp[j] = i;
					}
				}
			}
		}
	}

	template <typename T>
	void for_each_factor(int x, const T& t, int mul = 1) {
		if (x == 1) {
			t(mul);
		} else {
			int cur = mp[x], cnt = 0;
			while (x % cur == 0) {
				x /= cur;
				cnt++;
			}
			for_each_factor(x, t, mul);
			for (int i = 1; i <= cnt; i++) {
				mul *= cur;
				for_each_factor(x, t, mul);
			}
		}
	}
};

bool contains7(int x) {
	while (x) {
		if (x % 10 == 7) {
			return true;
		}
		x /= 10;
	}
	return false;
}

bool check(int x) {
	static Fact<maxn> fact;
	if (x % 7 == 0) {
		return false;
	}
	bool ans = true;
	fact.for_each_factor(x, [&](int f) -> void {
		if (ans && contains7(f)) {
			ans = false;
		}
	});
	return ans;
}

void solve() {
	int x;
	read_int(x);
	if (!check(x)) {
		cout << -1 << endl;
		return;
	}
	for (int i = x + 1;; i++) {
		if (check(i)) {
			cout << i << endl;
			return;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	init();
	int t = 1;
	read_int(t);
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}