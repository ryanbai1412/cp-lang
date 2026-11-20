#ifndef DEBUG
#pragma GCC optimize("O3,unroll-loops")
#endif

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

const long mod = 998244353;

struct Mat {
	int arr[2][2];

	friend Mat operator*=(Mat& a, const Mat& b) {
		long x[2][2]{};
		for (int i = 0; i < 2; i++) {
			for (int j = 0; j < 2; j++) {
				for (int k = 0; k < 2; k++) {
					x[i][k] += long(a.arr[i][j]) * b.arr[j][k];
				}
			}
		}
		for (int i = 0; i < 2; i++) {
			for (int j = 0; j < 2; j++) {
				a.arr[i][j] = x[i][j] % mod;
			}
		}
		return a;
	}

	friend Mat operator*(Mat a, const Mat& b) { return a *= b; }

	void flip() {
		swap(arr[0][0], arr[1][1]);
		swap(arr[0][1], arr[1][0]);
	}

	void reverse() { swap(arr[0][0], arr[1][1]); }
} mats[]{{1, 0, 1, 1}, {1, 1, 0, 1}};

struct Node {
	int x, sz, pri;
	bool flip, rev;
	Mat data;
	Node *l, *r;

	static Node from(int x) {
#ifdef DEBUG
		static mt19937 cowng(1412);
#else
		static mt19937 cowng(
			chrono::steady_clock::now().time_since_epoch().count());
#endif
		return {x, 1, cowng(), false, false, mats[x], nullptr, nullptr};
	}

	static Node* falloc(int x) {
		static Node* heap[100];
		static int hind = -1, csz = 128, cind = 128;

		if (csz == cind) {
			heap[++hind] = new Node[csz <<= 1];
			cind = 0;
		}

		Node* ptr = &heap[hind][cind++];
		*ptr = Node::from(x);

		return ptr;
	}

	void maintain() {
		sz = 1;
		data = mats[x];
		if (l) {
			sz += l->sz;
			data = l->data * data;
		}
		if (r) {
			sz += r->sz;
			data *= r->data;
		}

		if (flip) {
			data.flip();
		}
		if (rev) {
			data.reverse();
		}
	}

	void push() {
		if (flip) {
			x ^= 1;
		}
		if (rev) {
			swap(l, r);
		}
		if (l) {
			l->flip ^= flip;
			l->rev ^= rev;
			l->maintain();
		}
		if (r) {
			r->flip ^= flip;
			r->rev ^= rev;
			r->maintain();
		}
		flip = rev = false;
	}
};

using Treap = Node*;

namespace treap {

int size(Treap t) {
	return t ? t->sz : 0;
}

void split(Treap t, int ind, Treap& l, Treap& r) {
	if (!t) {
		l = r = nullptr;
		return;
	}

	t->push();
	if (ind <= size(t->l)) {
		split(t->l, ind, l, t->l);
		r = t;
	} else {
		split(t->r, ind - size(t->l) - 1, t->r, r);
		l = t;
	}
	t->maintain();
}

pair<Treap, Treap> split(Treap t, int ind) {
	Treap a{}, b{};
	split(t, ind, a, b);
	return {a, b};
}

tuple<Treap, Treap, Treap> split(Treap t, int l, int r) {
	Treap a{}, b{}, c{}, d{};
	split(t, r, c, d);
	split(c, l, a, b);
	return {a, b, d};
}

void merge(Treap& t, Treap l, Treap r) {
	if (!l || !r) {
		t = l ? l : r;
		return;
	}

	if (l->pri < r->pri) {
		l->push();
		merge(l->r, l->r, r);
		t = l;
	} else {
		r->push();
		merge(r->l, l, r->l);
		t = r;
	}
	t->maintain();
}

Treap merge(Treap l, Treap r) {
	Treap a{};
	merge(a, l, r);
	return a;
}

}  // namespace treap

void solve() {
	int n, q;
	cin >> n >> q;
	string s;
	cin >> s;
	Treap t{};
	for (auto& a : s) {
		t = treap::merge(t, Node::falloc(a == 'E'));
	}
	auto answer = [&]() -> void {
		Mat ans = t->data;
		long num = ans.arr[0][0] + ans.arr[0][1];
		long den = ans.arr[1][0] + ans.arr[1][1];
		if (num >= mod) {
			num -= mod;
		}
		if (den >= mod) {
			den -= mod;
		}
		cout << num << " " << den << endl;
	};
	answer();
	while (q--) {
		string type;
		cin >> type;
		if (type == "APPEND") {
			char c;
			cin >> c;
			t = treap::merge(t, Node::falloc(c == 'E'));
		} else if (type == "FLIP") {
			int l, r;
			cin >> l >> r;
			l--;
			auto [a, b, c] = treap::split(t, l, r);
			b->flip ^= true;
			b->maintain();
			t = treap::merge(a, b);
			t = treap::merge(t, c);
		} else {
			int l, r;
			cin >> l >> r;
			l--;
			auto [a, b, c] = treap::split(t, l, r);
			b->rev ^= true;
			b->maintain();
			t = treap::merge(a, b);
			t = treap::merge(t, c);
		}
		answer();
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}