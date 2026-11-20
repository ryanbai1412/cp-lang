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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct N {
    N *l, *r;
    int h;

    N(N* l, N* r) : l(l), r(r), h(hsh(l, r)) {}
    N(N* l, N* r, int h) : l(l), r(r), h(h) {}

    static int hsh(N* l, N* r) {
        static map<pair<int, int>, int> mp;
        static int hind = 1412;
        auto [it, inserted] = mp.insert({{l ? l->h : -1, r ? r->h : -1}, 0});
        if (inserted) {
            it->second = hind++;
        }
        return it->second;
    }
} S(nullptr, nullptr, 1), K(nullptr, nullptr, 2);

N* apply_c(N* o);

bool rule1(N*& o) {
    if (o->l && o->l->l == &K) {
        o = o->l->r;
        return true;
    }
    return false;
}

bool rule2(N*& o) {
    if (o->l && o->l->l && o->l->l->l == &S) {
        N *a = o->l->l->r, *b = o->l->r, *c = o->r;
        o = new N(new N(a, c), new N(b, c));
        return true;
    }
    return false;
}

bool rule3(N*& o) {
    N* l = apply_c(o->l);
    if (l != o->l) {
        o = new N(l, o->r);
        return true;
    }
    return false;
}

bool rule4(N*& o) {
    N* r = apply_c(o->r);
    if (r != o->r) {
        o = new N(o->l, r);
        return true;
    }
    return false;
}

N* apply(N* o) {
    if (rule1(o)) {
        dbg(o);
        return apply_c(o);
    }
    if (rule2(o)) {
        return apply_c(o);
    }
    if (rule3(o)) {
        return apply_c(o);
    }
    if (rule4(o)) {
        return apply_c(o);
    }
    return o;
}

N* apply_c(N* o) {
    if (!o) {
        return o;
    }
    return apply(o);
//    static map<int, N*> cache;
//    auto [it, inserted] = cache.emplace(o->h, nullptr);
//    if (inserted) {
//        it->second = apply(o);
//    }
//    if (!it->second) {
//        cout << (new int[5])[10];
//    }
//    assert(it->second);
//    return it->second;
}

struct P {
    string s;
    int ind = 0;

    P(string s) : s(std::move(s)) {}

    char next() {
        return s[ind++];
    }

    N* parse() {
        char c = next();
        if (c == 'S') {
            return &S;
        } else if (c == 'K') {
            return &K;
        }
        assert(c == '(');
        N *l = parse(), *r = parse();
        c = next();
        assert(c == ')');
        return new N(l, r);
    }
};

struct W {
    string s;

    void write(N* o) {
        if (!o) {
            return;
        } else if (o == &S) {
            s += 'S';
            return;
        } else if (o == &K) {
            s += 'K';
            return;
        }
        s += '(';
        write(o->l);
        write(o->r);
        s += ')';
    }
};

void solve(const string& s) {
    P parser(s);
    N* o = parser.parse();
    o = apply_c(o);
    W writer;
    writer.write(o);
    cout << writer.s << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    string s;
    while (cin >> s) {
        solve(s);
    }
}