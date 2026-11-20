
istream& operator>>(istream& in, i128& x) {
    i64 nx;
    in >> nx;
    x = nx;
    return in;
}
ostream& operator<<(ostream& out, i128 x) {
    return out << i64(x);
}
template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}
template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}
i128 gcd(i128 a, i128 b) {
    return __gcd(a, b);
}
long norm(long x, long m) {
    if (0 <= x && x < m) {
        return x;
    } else if (-m <= x && x < 0) {
        return x + m;
    }
    x %= m;
    if (x < 0) {
        return x + m;
    }
    return x;
}
void exgcd(long a, long b, long& x, long& y) {
    if (a > b) {
        exgcd(b, a, y, x);
        return;
    } else if (b == 1) {
        x = 0;
        y = 1;
        return;
    }
    long q = b / a, r = b % a;
    exgcd(a, r, x, y);
    x -= q * y;
}
long minv(long x, long m) {
    x = norm(x, m);
    if (gcd(x, m) != 1) {
        dbg(x, m);
    }
    assert(gcd(x, m) == 1);
    long a1, a2;
    exgcd(x, m, a1, a2);
    return norm(a1, m);
}
using P = pair<long, long>;
// P solve_coprime(const P& p1, const P& p2) {
//     auto [a1, b1] = p1;
//     auto [a2, b2] = p2;
//
//     a1 = norm(a1, b1);
//     a2 = norm(a2, b2);
//
//     long m = b1 * b2;
//
//     return {((a1 * b2 % m) * minv(b2, b1) + (a2 * b1 % m) * minv(b1, b2)) %
//     m,
//             m};
// }
P solve_crt(const P& p1, const P& p2, bool& fail) {
    auto [a1, b1] = p1;
    auto [a2, b2] = p2;
    a1 = norm(a1, b1);
    a2 = norm(a2, b2);
    long g = __gcd(b1, b2), m = b1 / g * b2;
    if (a1 % g != a2 % g) {
        fail = true;
        return p1;
    }
    return {((a1 % g) + b2 * minv(b2 / g, b1 / g) % m * (a1 / g) +
                b1 * minv(b1 / g, b2 / g) % m * (a2 / g)) % m,
            m};
}
long cdiv(long a, long b) {
    return (a + b - 1) / b;
}
void solve() {
    int n, m;
    cin >> n >> m;
    vector<array<long, 3>> arr(n);
    for (int i = 0; i < 3; i++) {
        for (auto& a : arr) {
            cin >> a[i];
        }
    }
    vector<long> iswords(m);
    for (auto& a : iswords) {
        cin >> a;
    }
    multiset swords(begin(iswords), end(iswords));
    long mn_x = 0;
    bool fail = false;
    vector<P> eqns;
    auto add_eqn = [&](long a, long b, long m) -> void {
        a = norm(a, m);
        b = norm(b, m);
        long g = gcd(a, m);
        if (!a) {
            if (b) {
                fail = true;
            }
            return;
        }
        if (b % g) {
            fail = true;
            return;
        }
        a /= g;
        b /= g;
        m /= g;
        eqns.emplace_back(minv(a, m) * b % m, m);
    };
    for (auto& [life, recovery, nsword] : arr) {
        auto it = swords.upper_bound(life);
        if (it != begin(swords)) {
            --it;
        }
        long csword = *it;
        mn_x = max(mn_x, cdiv(life, csword));
        add_eqn(csword, life, recovery);
        swords.erase(it);
        swords.insert(nsword);
    }
    eqns.emplace_back(0, 1);
    dbg(eqns);
    while (sz(eqns) >= 2) {
        auto p1 = eqns.back();
        eqns.pop_back();
        auto p2 = eqns.back();
        eqns.pop_back();
        eqns.push_back(solve_crt(p1, p2, fail));
    }
    if (fail) {
        cout << -1 << endl;
        return;
    }
    auto [v, mod] = eqns[0];
    dbg(v, mod, mn_x);
    long k = max(long(0), cdiv(mn_x - v, mod));
    cout << k * mod + v << endl;
}
