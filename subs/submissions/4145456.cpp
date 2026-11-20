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

void read(int& x) {
	x = (*bptr++) - '0';
	while ((c = (*bptr++)) >= '0') {
		x = x * 10 + c - '0';
	}
}
}  // namespace io

const int maxn = 1e5 + 5, bsize = 280, b1 = 800, bcnt = maxn / bsize + 1;

int cdiv(int a, int b) {
	return (a + b - 1) / b;
}

int n, q, arr[maxn], cnt[maxn], xcnt[maxn], comp[maxn], icomp[maxn];
long ans = 0;

void add(int i) {
	int x = arr[i];
	ans = max(ans, long(++cnt[x]) << 32 | x);
}

void rm(int i) {
	dbg("RM", i);
	cnt[arr[i]]--;
}

void solve1() {
	dbg("s1");
	int ind = 0;
	for (int i = 0; i < maxn; i++) {
		if (xcnt[i]) {
			icomp[ind] = i;
			comp[i] = ind++;
		}
	}
	for (int i = 0; i < n; i++) {
		arr[i] = comp[arr[i]];
	}
	int queries[q][2];
	for (auto& [l, r] : queries) {
		io::read(l);
		io::read(r);
		l--;
		r--;
	}
	long mans[q] {};
	for (int i = 0; i < ind; i++) {
		int psum[n + 1];
		psum[0] = 0;
		for (int j = 0; j < n; j++) {
			psum[j + 1] = psum[j] + (arr[j] == i);
		}
		for (int j = 0; j < q; j++) {
			auto &[l, r] = queries[j];
			mans[j] = max(mans[j], long(psum[r + 1] - psum[l]) << 32 | i);
		}
	}
	for (auto& a : mans) {
		cout << icomp[a & 0xFFFFFFFF] << endl;
	}
}

void solve() {
	int ccnt = 0;
	io::read(n);
	io::read(q);
	for (int i = 0; i < n; i++) {
		io::read(arr[i]);
		if (!xcnt[arr[i]]) {
			ccnt++;
			xcnt[arr[i]]++;
		}
	}
	if (ccnt <= b1) {
		solve1();
		return;
	}
	vector<array<int, 3>> queries[bcnt];
	int mxn = 0;
	long qans[q];
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
			ans = 0;
		} else {
			int lbkt = cdiv(l, bsize);
			dbg(lbkt);
			queries[lbkt].push_back({l, r, i});
			mxn = max(mxn, lbkt);
		}
	}
	for (int i = 0; i <= mxn; i++) {
		sort(begin(queries[i]), end(queries[i]), [&](const auto& a, const auto& b) -> bool { return a[1] < b[1]; });
		memset(cnt, 0, sizeof(cnt));
		ans = 0;
		int cl = i * bsize, cr = cl - 1;
		for (auto& [l, r, qind] : queries[i]) {
			while (cr < r) {
				add(++cr);
			}
			long cans = ans;
			for (int j = l; j < cl; j++) {
				add(j);
			}
			qans[qind] = ans;
			for (int j = l; j < cl; j++) {
				rm(j);
			}
			ans = cans;
		}
	}
	for (auto& a : qans) {
		cout << (a & 0xFFFFFFFF) << endl;
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