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

namespace io {
const int bufsz = 2 << 20;
char buf[bufsz], *bptr = buf, c;

void init() {
	fread(buf, 1, bufsz, stdin);
}

void read(int &x) {
	x = (*bptr++) - '0';
	while ((c = (*bptr++)) >= '0') {
		x = x * 10 + c - '0';
	}
}
}

const int maxn = 1e5 + 5, bsize = 400, bcnt = maxn / bsize + 1;

int cdiv(int a, int b) {
	return (a + b - 1) / b;
}

int arr[maxn], cnt[maxn], ans = 0, ans_cnt = 0;

void add(int i) {
	int x = arr[i];
	if (++cnt[x] > ans_cnt) {
		ans = x;
		ans_cnt = cnt[x];
	} else if (cnt[x] == ans_cnt && x >= ans) {
		ans = x;
	}
}

void rm(int i) {
	dbg("RM", i);
	cnt[arr[i]]--;
}

void solve() {
	int n, q;
	io::read(n);
	io::read(q);
	for (int i = 0; i < n; i++) {
		io::read(arr[i]);
	}
	vector<array<int, 3>> queries[bcnt];
	int mxn = 0;
	int qans[q];
	for (int i = 0; i < q; i++) {
		int l, r;
		io::read(l);
		io::read(r);
		l--;
		r--;
		if ((r - l + 1) <= bsize) {
			for (int j = l; j <= r; j++) {
				add(j);
			}
			qans[i] = ans;
			for (int j = l; j <= r; j++) {
				rm(j);
			}
			ans = ans_cnt = 0;
		} else {
			int lbkt = cdiv(l, bsize);
			dbg(lbkt);
			queries[lbkt].push_back({l, r, i});
			mxn = max(mxn, lbkt);
		}
	}
	for (int i = 0; i <= mxn; i++) {
		sort(begin(queries[i]), end(queries[i]), [&](const auto &a, const auto &b) -> bool {
			return a[1] < b[1];
		});
		memset(cnt, 0, sizeof(cnt));
		ans = ans_cnt = 0;
		int cl = i * bsize, cr = cl - 1;
		for (auto& [l, r, qind] : queries[i]) {
			while (cr < r) {
				add(++cr);
			}
			int cans = ans, cans_cnt = ans_cnt;
			for (int j = l; j < cl; j++) {
				add(j);
			}
			qans[qind] = ans;
			for (int j = l; j < cl; j++) {
				rm(j);
			}
			ans = cans;
			ans_cnt = cans_cnt;
		}
	}
	for (auto& a : qans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	io::init();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}