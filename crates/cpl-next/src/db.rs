//! The query database (handbook/design/next/incremental.md, "The rule" and "Early cutoff"): the
//! snapshot sets inputs, and each query is a pure fn of the inputs and queries it reads. An
//! answer is kept with what it read and recomputed only when one of those changed since it was
//! last verified; a recomputed answer equal to the old one changes nothing downstream.
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// A query: a pure fn from a key to an answer, computed through `Db::get` alone.
pub trait Query: 'static {
    type Key: Clone + Eq + Hash + std::fmt::Debug + Send + Sync + 'static;
    type Value: Clone + Eq + Send + Sync + 'static;
    /// The name a count of recomputations reports.
    const NAME: &'static str;
    fn compute(db: &Db, key: &Self::Key) -> Self::Value;
    /// The answer for a key met again while it is being computed; `None` for a query whose
    /// answers never depend on themselves.
    fn recover(_key: &Self::Key) -> Option<Self::Value> {
        None
    }
}

/// An input: a key the snapshot sets, read like a query.
pub trait Input: 'static {
    type Key: Clone + Eq + Hash + std::fmt::Debug + Send + Sync + 'static;
    type Value: Clone + Eq + Send + Sync + 'static;
}

type Dep = Arc<dyn Fn(&Db) -> u64 + Send + Sync>;

struct Memo<V> {
    value: V,
    deps: Vec<Dep>,
    verified: u64,
    changed: u64,
}

#[derive(Default)]
struct State {
    revision: u64,
    /// By query or input type: `HashMap<Key, Memo<Value>>`.
    memos: HashMap<TypeId, Box<dyn Any + Send>>,
    /// What each query being computed has read so far, innermost last.
    frames: Vec<Vec<Dep>>,
    active: Vec<(TypeId, Box<dyn Any + Send>)>,
    computed: HashMap<&'static str, usize>,
}

#[derive(Default)]
pub struct Db {
    state: Mutex<State>,
}

struct InputMarker<I>(std::marker::PhantomData<I>);

impl Db {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn table<K: Eq + Hash + Send + 'static, V: Send + 'static>(
        st: &mut State,
        t: TypeId,
    ) -> &mut HashMap<K, Memo<V>> {
        (st.memos
            .entry(t)
            .or_insert_with(|| Box::new(HashMap::<K, Memo<V>>::new())))
        .downcast_mut()
        .expect("a type's table holds that type's keys and answers")
    }

    /// Sets input `key`. A value equal to the one held changes nothing.
    pub fn set<I: Input>(&self, key: I::Key, value: I::Value) {
        let mut st = self.lock();
        let t = TypeId::of::<InputMarker<I>>();
        let rev = st.revision + 1;
        let table = Self::table::<I::Key, I::Value>(&mut st, t);
        if table.get(&key).is_some_and(|m| m.value == value) {
            return;
        }
        table.insert(
            key,
            Memo {
                value,
                deps: Vec::new(),
                verified: rev,
                changed: rev,
            },
        );
        st.revision = rev;
    }

    /// Input `key`'s value, read by the query being computed.
    pub fn input<I: Input>(&self, key: &I::Key) -> Option<I::Value> {
        let t = TypeId::of::<InputMarker<I>>();
        let mut st = self.lock();
        let value = (Self::table::<I::Key, I::Value>(&mut st, t).get(key)).map(|m| m.value.clone());
        let k = key.clone();
        let dep: Dep = Arc::new(move |db: &Db| {
            let mut st = db.lock();
            (Self::table::<I::Key, I::Value>(&mut st, TypeId::of::<InputMarker<I>>()).get(&k))
                .map_or(u64::MAX, |m| m.changed)
        });
        if let Some(f) = st.frames.last_mut() {
            f.push(dep);
        }
        value
    }

    /// Query `Q`'s answer for `key`, recomputed only if something it read changed.
    pub fn get<Q: Query>(&self, key: &Q::Key) -> Q::Value {
        let (value, _) = self.fetch::<Q>(key);
        let k = key.clone();
        let dep: Dep = Arc::new(move |db: &Db| db.fetch::<Q>(&k).1);
        if let Some(f) = self.lock().frames.last_mut() {
            f.push(dep);
        }
        value
    }

    /// How many times each query was computed, by name.
    pub fn computed(&self) -> HashMap<&'static str, usize> {
        self.lock().computed.clone()
    }

    /// The answer and the revision it last changed in.
    fn fetch<Q: Query>(&self, key: &Q::Key) -> (Q::Value, u64) {
        let t = TypeId::of::<Q>();
        let (rev, held) = {
            let mut st = self.lock();
            if (st.active.iter()).any(|(a, k)| *a == t && k.downcast_ref::<Q::Key>() == Some(key)) {
                drop(st);
                let value = Q::recover(key)
                    .unwrap_or_else(|| panic!("query {}({key:?}) reads its own answer", Q::NAME));
                return (value, u64::MAX);
            }
            let rev = st.revision;
            let held = (Self::table::<Q::Key, Q::Value>(&mut st, t).get(key))
                .map(|m| (m.value.clone(), m.deps.clone(), m.verified, m.changed));
            (rev, held)
        };
        if let Some((value, deps, verified, changed)) = &held {
            if *verified == rev {
                return (value.clone(), *changed);
            }
            self.lock().active.push((t, Box::new(key.clone())));
            let fresh = deps.iter().all(|d| d(self) <= *verified);
            self.lock().active.pop();
            if fresh {
                let mut st = self.lock();
                if let Some(m) = Self::table::<Q::Key, Q::Value>(&mut st, t).get_mut(key) {
                    m.verified = rev;
                }
                return (value.clone(), *changed);
            }
        }
        {
            let mut st = self.lock();
            st.frames.push(Vec::new());
            st.active.push((t, Box::new(key.clone())));
            *st.computed.entry(Q::NAME).or_default() += 1;
        }
        let value = Q::compute(self, key);
        let mut st = self.lock();
        st.active.pop();
        let deps = st.frames.pop().unwrap_or_default();
        let changed = match &held {
            Some((old, _, _, changed)) if *old == value => *changed,
            _ => rev,
        };
        Self::table::<Q::Key, Q::Value>(&mut st, t).insert(
            key.clone(),
            Memo {
                value: value.clone(),
                deps,
                verified: rev,
                changed,
            },
        );
        (value, changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Num;
    impl Input for Num {
        type Key = u32;
        type Value = i64;
    }
    struct Total;
    impl Query for Total {
        type Key = ();
        type Value = i64;
        const NAME: &'static str = "total";
        fn compute(db: &Db, _: &()) -> i64 {
            db.input::<Num>(&1).unwrap_or(0) + db.input::<Num>(&2).unwrap_or(0)
        }
    }
    struct Twice;
    impl Query for Twice {
        type Key = ();
        type Value = i64;
        const NAME: &'static str = "twice";
        fn compute(db: &Db, _: &()) -> i64 {
            db.get::<Total>(&()) * 2
        }
    }
    struct Loop;
    impl Query for Loop {
        type Key = u32;
        type Value = u32;
        const NAME: &'static str = "loop";
        fn compute(db: &Db, k: &u32) -> u32 {
            db.get::<Loop>(&(1 - k)) + 1
        }
        fn recover(_: &u32) -> Option<u32> {
            Some(0)
        }
    }

    /// An answer is recomputed only when what it read changed, and one whose new answer equals
    /// the old stops there: its readers are not recomputed.
    #[test]
    fn a_query_reruns_only_when_what_it_read_changed() {
        let db = Db::default();
        db.set::<Num>(1, 2);
        db.set::<Num>(2, 3);
        assert_eq!(db.get::<Twice>(&()), 10);
        assert_eq!(db.get::<Twice>(&()), 10);
        let n = |db: &Db, q| db.computed().get(q).copied().unwrap_or(0);
        assert_eq!((n(&db, "total"), n(&db, "twice")), (1, 1));
        db.set::<Num>(1, 2);
        assert_eq!(db.get::<Twice>(&()), 10);
        assert_eq!((n(&db, "total"), n(&db, "twice")), (1, 1));
        db.set::<Num>(1, 3);
        db.set::<Num>(2, 2);
        assert_eq!(db.get::<Twice>(&()), 10);
        assert_eq!((n(&db, "total"), n(&db, "twice")), (2, 1));
        db.set::<Num>(2, 4);
        assert_eq!(db.get::<Twice>(&()), 14);
        assert_eq!((n(&db, "total"), n(&db, "twice")), (3, 2));
    }

    /// A key met again while it is computed takes the query's recovery answer.
    #[test]
    fn a_cycle_takes_the_recovery_answer() {
        let db = Db::default();
        assert_eq!(db.get::<Loop>(&0), 2);
    }
}
