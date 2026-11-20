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

struct NodeData {
	int sz, sum, pre, suf, ans;

	static NodeData from(int x, int cnt) {
		int tot = x * cnt;
		return {cnt, tot, tot, tot, tot};
	}

	friend NodeData operator+=(NodeData& a, const NodeData& b) {
		a.ans = max({a.ans, b.ans, a.suf + b.pre});
		a.pre = max(a.pre, a.sum + b.pre);
		a.suf = max(a.suf + b.sum, b.suf);
		a.sz += b.sz;
		a.sum += b.sum;
		return a;
	}

	friend NodeData operator+(NodeData a, const NodeData& b) { return a += b; }

	void reverse() { swap(pre, suf); }
};

struct Node {
	int x, pri, set;
	bool rev;
	NodeData data;
	Node *l, *r;

	static Node from(int x) {
		static mt19937 cowng(
#ifdef DEBUG
			1412
#else
			chrono::steady_clock::now().time_since_epoch().count()
#endif
		);
		return {x, cowng(), -1, false, NodeData::from(x, 1), nullptr, nullptr};
	}

	static Node* falloc(int x) {
		static Node* heap[100];
		static int hind = -1, csz = 128, cptr = 128;

		if (csz == cptr) {
			heap[++hind] = new Node[csz <<= 1];
			cptr = 0;
		}

		Node* ptr = &heap[hind][cptr++];
		*ptr = Node::from(x);

		return ptr;
	}

	void maintain() {
		if (set != -1) {
			int sz = 1;
			if (l) {
				sz += l->data.sz;
			}
			if (r) {
				sz += r->data.sz;
			}
			data = NodeData::from(set, sz);
		} else {
			data = NodeData::from(x, 1);
			if (l) {
				data = l->data + data;
			}
			if (r) {
				data += r->data;
			}
			if (rev) {
				data.reverse();
			}
		}
	}

	void push() {
		if (set != -1) {
			x = set;
		}
		if (rev) {
			swap(l, r);
		}
		if (set != -1) {
			if (l) {
				l->set = set;
				l->rev = rev;
			}
			if (r) {
				r->set = set;
				r->rev = rev;
			}
		} else {
			if (l) {
				l->rev ^= rev;
			}
			if (r) {
				r->rev ^= rev;
			}
		}
		set = -1;
		rev = false;
		if (l) {
			l->maintain();
		}
		if (r) {
			r->maintain();
		}
	}

	void reverse() {
		rev ^= true;
		maintain();
	}

	void setv(int v) {
		set = v;
		rev = false;
		maintain();
	}
};

using Treap = Node*;

namespace treap {

int size(Treap t) {
	return t ? t->data.sz : 0;
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

void merge(Treap &t, Treap l, Treap r) {
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
	Treap t{};
	for (int i = 0; i < n; i++) {
		int x;
		cin >> x;
		t = treap::merge(t, Node::falloc(x));
	}
	while(q--) {
		string type;
		cin >> type;
		if (type == "INSERT") {
			int ind, cnt;
			cin >> ind >> cnt;
			auto [a, b] = treap::split(t, ind);
			for (int i = 0; i < cnt; i++) {
				int x;
				cin >> x;
				a = treap::merge(a, Node::falloc(x));
			}
			t = treap::merge(a, b);
		} else if (type == "MAX-SUM") {
			cout << t->data.ans << endl;
		} else {
			int ind, cnt;
			cin >> ind >> cnt;
			ind--;
			auto [a, b, c] = treap::split(t, ind, ind + cnt);
			if (type == "DELETE") {
				t = treap::merge(a, c);
			} else if (type == "MAKE-SAME") {
				int x;
				cin >> x;
				b->setv(x);
				t = treap::merge(a, treap::merge(b, c));
			} else if (type == "REVERSE") {
				b->reverse();
				t = treap::merge(a, treap::merge(b, c));
			} else {
				cout << b->data.sum << endl;
				t = treap::merge(a, treap::merge(b, c));
			}
		}
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