#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int(x.size())

const int maxn = 1024, tmaxn = maxn * 2 + 1;

int n;
int v[tmaxn][tmaxn];

void update2(int xo, int o, int l, int r, int ind, int x) {
	v[xo][o] += x;
	if(l < r) {
		int mid = (l + r) / 2;
		if(ind <= mid) {
			update2(xo, o * 2, l, mid, ind, x);
		}else {
			update2(xo, o * 2 + 1, mid + 1, r, ind, x);
		}
	}
}

void update1(int o, int l, int r, int i, int j, int x) {
	update2(o, 1, 0, n - 1, j, x);
	if(l < r) {
		int mid = (l + r) / 2;
		if(i <= mid) {
			update1(o * 2, l, mid, i, j, x);
		}else {
			update1(o * 2 + 1, mid + 1, r, i, j, x);
		}
	}
}

void update(int i, int j, int x) {
	update1(1, 0, n - 1, i, j, x);
}

int query2(int xo, int o, int l, int r, int ql, int qr) {
	if(ql <= l && r <= qr) {
		return v[xo][o];
	}else {
		int ans = 0;
		int mid = (l + r) / 2;
		if(ql <= mid) {
			ans += query2(xo, o * 2, l, mid, ql, qr);
		}
		if(qr > mid) {
			ans += query2(xo, o * 2 + 1, mid + 1, r, ql, qr);
		}
		return ans;
	}
}

int query1(int o, int l, int r, int x1, int y1, int x2, int y2) {
	if(x1 <= l && r <= x2) {
		return query2(o, 1, 0, n - 1, y1, y2);
	}else {
		int ans = 0;
		int mid = (l + r) / 2;
		if(x1 <= mid) {
			ans += query1(o * 2, l, mid, x1, y1, x2, y2);
		}
		if(x2 > mid) {
			ans += query1(o * 2 + 1, mid + 1, r, x1, y1, x2, y2);
		}
		return ans;
	}
}

int query(int x1, int y1, int x2, int y2) {
	return query1(1, 0, n - 1, x1, y1, x2, y2);
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	while(true) {
		int type;
		cin >> type;
		if(type == 0) {
			cin >> n;
		}else if(type == 1) {
			int i, j, x;
			cin >> i >> j >> x;
			update(i, j, x);
		}else if(type == 2) {
			int x1, y1, x2, y2;
			cin >> x1 >> y1 >> x2 >> y2;
			cout << query(x1, y1, x2, y2) << endl;
		}else {
			break;
		}
	}
}