
struct FastCin {
    char buf[20 << 20], *i = buf;
    FastCin() {
        std::ignore = fread_unlocked(buf, 1, sizeof(buf), stdin);
    }
    FastCin& operator>>(int& x) {
        while (*i < '-') {
            i++;
        }
        bool neg = false;
        if (*i == '-') {
            neg = true;
            i++;
        }
        x = 0;
        while (*i >= '0') {
            x = x * 10 + (*(i++) - '0');
        }
        if (neg) {
            x = -x;
        }
        return *this;
    }
} in;
u32 all_mask(u32 x) {
    if (x == 32) {
        return -1;
    }
    return (1 << x) - 1;
}
pair<u32, u32> split_u64(u64 x) {
    return {x >> 32, x & u32(-1)};
}
bool on(u64 mask, int ind) {
    return (mask >> ind) & 1;
}
struct Solver {
    struct Seg {
        mutable int l, r;
        bool operator<(const Seg& s) const {
            return l < s.l;
        }
    };
    map<Seg, u32> s {{{0, int(1e9)}, 0}};
    using It = decltype(s)::iterator;
    void split_one(It& it) {
        auto& [ql, qr] = it->first;
        u32 v = it->second;
        if (ql + 1 == qr) {
            return;
        }
        ql++;
        it = s.insert(it, {{ql - 1, ql}, v});
        // s.erase(it);
        // it = s.insert({{ql, ql + 1}, v}).first;
        // s[{ql + 1, qr}] = v;
    }
    void add(It& it, u32 x) {
        if (!x) {
            return;
        } else if (x == 1) {
            if (it->second == u32(-1)) {
                it->second = 0;
                auto m_it = it;
                add(++it, 1);
                auto n_it = next(m_it);
                if (!n_it->second) {
                    int ql = m_it->first.l;
                    s.erase(m_it);
                    n_it->first.l = ql;
                }
                return;
            } else {
                split_one(it);
                it->second++;
                return;
            }
        }
        split_one(it);
        auto [top, bot] = split_u64(u64(it->second) + x);
        it->second = bot;
        add(++it, top);
    }
    void sub(It& it, u32 x) {
        if (!x) {
            return;
        } else if (x == 1) {
            if (!it->second) {
                it->second = u32(-1);
                auto m_it = it;
                sub(++it, 1);
                auto n_it = next(m_it);
                if (n_it->second == u32(-1)) {
                    int ql = m_it->first.l;
                    s.erase(m_it);
                    n_it->first.l = ql;
                }
                return;
            } else {
                split_one(it);
                it->second--;
                return;
            }
        }
        split_one(it);
        auto [top, bot] = split_u64((u64(1) << 63) + it->second - x);
        it->second = bot;
        sub(++it, (u32(1) << 31) - top);
    }
    It find_containing(int ind) {
        auto it = s.lower_bound({ind + 1, -1});
        return --it;
    }
    void update_aligned(bool neg, int ind, u32 x) {
        auto it = find_containing(ind);
        auto [ql, qr] = it->first;
        u32 v = it->second;
        assert(ql <= ind && ind < qr);
        if (ql < ind) {
            it->first.l = ind;
            s.insert(it, {{ql, ind}, v});
            // s.erase(it);
            // s[{ql, ind}] = v;
            // it = s.insert({{ind, qr}, v}).first;
        }
        if (neg) {
            sub(it, x);
        } else {
            add(it, x);
        }
    }
    void update(int ind, int x) {
        int bot = 32 - (ind % 32);
        bool neg = x < 0;
        if (neg) {
            x = -x;
        }
        update_aligned(neg, ind / 32, (x & all_mask(bot)) << (ind % 32));
        if (bot < 32) {
            update_aligned(neg, ind / 32 + 1, x >> bot);
        }
    }
    bool query(int ind) {
        auto it = find_containing(ind / 32);
        return on(it->second, ind % 32);
    }
};
void solve() {
    int q, tc1, tc2, tc3;
    in >> q >> tc1 >> tc2 >> tc3;
    Solver solver;
    while (q--) {
        int ty;
        in >> ty;
        if (ty == 1) {
            int x, ind;
            in >> x >> ind;
            solver.update(ind, x);
        } else {
            int ind;
            in >> ind;
            cout << solver.query(ind) << endl;
        }
    }
}
