//! The query database (handbook/design/next/incremental.md, "The rule" and "Early cutoff"): the
//! snapshot sets inputs, and each query is a pure fn of the inputs and queries it reads. An
//! answer is kept with what it read and recomputed only when one of those changed since it was
//! last verified; a recomputed answer equal to the old one changes nothing downstream.
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

/// Why the database could not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbError {
    /// A query met its own key while computing it and defines no answer for that cycle.
    Cycle { query: &'static str, key: String },
    /// A table held answers of another type than its query's.
    Table { of: &'static str },
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Cycle { query, key } => write!(f, "query {query}({key}) reads its own answer"),
            DbError::Table { of } => write!(f, "the table of {of} holds another type"),
        }
    }
}

/// A query: a pure fn from a key to an answer, computed through `Db::get` alone.
pub trait Query: 'static {
    type Key: Clone + Eq + Hash + std::fmt::Debug + Send + Sync + 'static;
    type Value: Clone + Eq + Send + Sync + 'static;
    /// The name a count of recomputations reports.
    const NAME: &'static str;
    fn compute(db: &Db, key: &Self::Key) -> Result<Self::Value, DbError>;
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

/// The revision a read answer last changed in; `u64::MAX` when it can no longer be read.
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
    /// Per thread, the queries it is computing: threads that share one database each compute
    /// their own, so one thread's reads never land in another's frame.
    threads: HashMap<ThreadId, Stack>,
    computed: HashMap<&'static str, usize>,
}

/// The queries one thread is computing, innermost last.
#[derive(Default)]
struct Stack {
    /// What each has read so far.
    frames: Vec<Vec<Dep>>,
    active: Vec<(TypeId, Box<dyn Any + Send>)>,
}

impl State {
    fn stack(&mut self) -> &mut Stack {
        self.threads.entry(std::thread::current().id()).or_default()
    }
    /// The frame of the query this thread is computing, if any.
    fn frame(&mut self) -> Option<&mut Vec<Dep>> {
        let me = std::thread::current().id();
        self.threads.get_mut(&me)?.frames.last_mut()
    }
    /// Ends this thread's innermost query, forgetting the thread once it computes none.
    fn pop(&mut self) -> Vec<Dep> {
        let me = std::thread::current().id();
        let Some(stack) = self.threads.get_mut(&me) else {
            return Vec::new();
        };
        stack.active.pop();
        let deps = stack.frames.pop().unwrap_or_default();
        if stack.active.is_empty() && stack.frames.is_empty() {
            self.threads.remove(&me);
        }
        deps
    }
}

#[derive(Default)]
pub struct Db {
    state: Mutex<State>,
}

struct InputMarker<I>(std::marker::PhantomData<I>);

type Answer<Q> = Result<<Q as Query>::Value, DbError>;

impl Db {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn table<K: Eq + Hash + Send + 'static, V: Send + 'static>(
        st: &mut State,
        t: TypeId,
    ) -> Result<&mut HashMap<K, Memo<V>>, DbError> {
        (st.memos.entry(t))
            .or_insert_with(|| Box::new(HashMap::<K, Memo<V>>::new()))
            .downcast_mut()
            .ok_or(DbError::Table {
                of: std::any::type_name::<V>(),
            })
    }

    /// Sets input `key`. A value equal to the one held changes nothing.
    pub fn set<I: Input>(&self, key: I::Key, value: I::Value) -> Result<(), DbError> {
        let mut st = self.lock();
        let t = TypeId::of::<InputMarker<I>>();
        let rev = st.revision + 1;
        let table = Self::table::<I::Key, I::Value>(&mut st, t)?;
        if table.get(&key).is_some_and(|m| m.value == value) {
            return Ok(());
        }
        let memo = Memo {
            value,
            deps: Vec::new(),
            verified: rev,
            changed: rev,
        };
        table.insert(key, memo);
        st.revision = rev;
        Ok(())
    }

    /// Removes input `key`: a query that read it reads it as absent.
    pub fn remove<I: Input>(&self, key: &I::Key) -> Result<(), DbError> {
        let mut st = self.lock();
        let t = TypeId::of::<InputMarker<I>>();
        if Self::table::<I::Key, I::Value>(&mut st, t)?
            .remove(key)
            .is_some()
        {
            st.revision += 1;
        }
        Ok(())
    }

    /// Input `key`'s value, read by the query being computed.
    pub fn input<I: Input>(&self, key: &I::Key) -> Result<Option<I::Value>, DbError> {
        let t = TypeId::of::<InputMarker<I>>();
        let mut st = self.lock();
        let value =
            (Self::table::<I::Key, I::Value>(&mut st, t)?.get(key)).map(|m| m.value.clone());
        let k = key.clone();
        let dep: Dep = Arc::new(move |db: &Db| {
            let mut st = db.lock();
            let table = Self::table::<I::Key, I::Value>(&mut st, TypeId::of::<InputMarker<I>>());
            (table.ok().and_then(|t| t.get(&k))).map_or(u64::MAX, |m| m.changed)
        });
        if let Some(f) = st.frame() {
            f.push(dep);
        }
        Ok(value)
    }

    /// Query `Q`'s answer for `key`, recomputed only if something it read changed.
    pub fn get<Q: Query>(&self, key: &Q::Key) -> Answer<Q> {
        let (value, _) = self.fetch::<Q>(key);
        let k = key.clone();
        let dep: Dep = Arc::new(move |db: &Db| db.fetch::<Q>(&k).1);
        if let Some(f) = self.lock().frame() {
            f.push(dep);
        }
        value
    }

    /// How many times each query was computed, by name.
    pub fn computed(&self) -> HashMap<&'static str, usize> {
        self.lock().computed.clone()
    }

    /// The answer and the revision it last changed in.
    fn fetch<Q: Query>(&self, key: &Q::Key) -> (Answer<Q>, u64) {
        let t = TypeId::of::<Q>();
        let (rev, held) = {
            let mut st = self.lock();
            if (st.stack().active.iter())
                .any(|(a, k)| *a == t && k.downcast_ref::<Q::Key>() == Some(key))
            {
                drop(st);
                let cycle = DbError::Cycle {
                    query: Q::NAME,
                    key: format!("{key:?}"),
                };
                return (Q::recover(key).ok_or(cycle), u64::MAX);
            }
            let rev = st.revision;
            let held = match Self::table::<Q::Key, Answer<Q>>(&mut st, t) {
                Ok(table) => (table.get(key))
                    .map(|m| (m.value.clone(), m.deps.clone(), m.verified, m.changed)),
                Err(e) => return (Err(e), u64::MAX),
            };
            (rev, held)
        };
        if let Some((value, deps, verified, changed)) = &held {
            if *verified == rev {
                return (value.clone(), *changed);
            }
            let mut st = self.lock();
            st.stack().frames.push(Vec::new());
            st.stack().active.push((t, Box::new(key.clone())));
            drop(st);
            let fresh = deps.iter().all(|d| d(self) <= *verified);
            self.lock().pop();
            if fresh {
                let mut st = self.lock();
                if let Ok(Some(m)) =
                    Self::table::<Q::Key, Answer<Q>>(&mut st, t).map(|t| t.get_mut(key))
                {
                    m.verified = rev;
                }
                return (value.clone(), *changed);
            }
        }
        {
            let mut st = self.lock();
            st.stack().frames.push(Vec::new());
            st.stack().active.push((t, Box::new(key.clone())));
            *st.computed.entry(Q::NAME).or_default() += 1;
        }
        let value = Q::compute(self, key);
        let mut st = self.lock();
        let deps = st.pop();
        let changed = match &held {
            Some((old, _, _, changed)) if *old == value => *changed,
            _ => rev,
        };
        let memo = Memo {
            value: value.clone(),
            deps,
            verified: rev,
            changed,
        };
        if let Err(e) =
            Self::table::<Q::Key, Answer<Q>>(&mut st, t).map(|t| t.insert(key.clone(), memo))
        {
            return (Err(e), u64::MAX);
        }
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
        fn compute(db: &Db, _: &()) -> Result<i64, DbError> {
            Ok(db.input::<Num>(&1)?.unwrap_or(0) + db.input::<Num>(&2)?.unwrap_or(0))
        }
    }
    struct Twice;
    impl Query for Twice {
        type Key = ();
        type Value = i64;
        const NAME: &'static str = "twice";
        fn compute(db: &Db, _: &()) -> Result<i64, DbError> {
            Ok(db.get::<Total>(&())? * 2)
        }
    }
    struct Loop;
    impl Query for Loop {
        type Key = u32;
        type Value = u32;
        const NAME: &'static str = "loop";
        fn compute(db: &Db, k: &u32) -> Result<u32, DbError> {
            Ok(db.get::<Loop>(&(k ^ 1))? + 1)
        }
        /// Keys 0 and 1 meet each other with an answer for the cycle; 2 and 3 with none.
        fn recover(k: &u32) -> Option<u32> {
            (*k < 2).then_some(0)
        }
    }

    #[test]
    fn a_removed_input_is_read_as_absent() {
        let db = Db::default();
        db.set::<Num>(1, 5).unwrap();
        db.set::<Num>(2, 7).unwrap();
        assert_eq!(db.get::<Total>(&()), Ok(12));
        db.remove::<Num>(&2).unwrap();
        assert_eq!(db.get::<Total>(&()), Ok(5));
        db.remove::<Num>(&2).unwrap();
        assert_eq!(db.get::<Total>(&()), Ok(5));
        assert_eq!(db.computed().get(Total::NAME), Some(&2));
    }

    /// An answer is recomputed only when what it read changed, and one whose new answer equals
    /// the old stops there: its readers are not recomputed.
    #[test]
    fn a_query_reruns_only_when_what_it_read_changed() {
        let db = Db::default();
        assert_eq!(db.set::<Num>(1, 2), Ok(()));
        assert_eq!(db.set::<Num>(2, 3), Ok(()));
        assert_eq!(db.get::<Twice>(&()), Ok(10));
        assert_eq!(db.get::<Twice>(&()), Ok(10));
        let n = |db: &Db, q| db.computed().get(q).copied().unwrap_or(0);
        assert_eq!((n(&db, "total"), n(&db, "twice")), (1, 1));
        assert_eq!(db.set::<Num>(1, 2), Ok(()));
        assert_eq!(db.get::<Twice>(&()), Ok(10));
        assert_eq!((n(&db, "total"), n(&db, "twice")), (1, 1));
        assert_eq!(db.set::<Num>(1, 3), Ok(()));
        assert_eq!(db.set::<Num>(2, 2), Ok(()));
        assert_eq!(db.get::<Twice>(&()), Ok(10));
        assert_eq!((n(&db, "total"), n(&db, "twice")), (2, 1));
        assert_eq!(db.set::<Num>(2, 4), Ok(()));
        assert_eq!(db.get::<Twice>(&()), Ok(14));
        assert_eq!((n(&db, "total"), n(&db, "twice")), (3, 2));
    }

    /// A key met again while it is computed takes the query's recovery answer.
    #[test]
    fn a_cycle_takes_the_recovery_answer() {
        let db = Db::default();
        assert_eq!(db.get::<Loop>(&0), Ok(2));
    }

    /// A cycle through a query that defines no answer for one is an error, not a stop.
    #[test]
    fn a_cycle_without_a_recovery_answer_is_an_error() {
        let db = Db::default();
        let cycle = DbError::Cycle {
            query: "loop",
            key: "2".into(),
        };
        assert_eq!(db.get::<Loop>(&2), Err(cycle));
    }

    /// Holds its first caller inside `compute` until a second thread has read the same key.
    struct Held;
    static ENTERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    static RELEASED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    impl Query for Held {
        type Key = ();
        type Value = u32;
        const NAME: &'static str = "held";
        fn compute(_: &Db, _: &()) -> Result<u32, DbError> {
            use std::sync::atomic::Ordering::SeqCst;
            if !ENTERED.swap(true, SeqCst) {
                while !RELEASED.load(SeqCst) {
                    std::thread::yield_now();
                }
            }
            Ok(7)
        }
    }

    /// A key one thread is computing is not a cycle for another thread that reads it.
    #[test]
    fn another_threads_query_is_not_a_cycle() {
        use std::sync::atomic::Ordering::SeqCst;
        let db = Db::default();
        std::thread::scope(|scope| {
            let first = scope.spawn(|| db.get::<Held>(&()));
            while !ENTERED.load(SeqCst) {
                std::thread::yield_now();
            }
            let second = db.get::<Held>(&());
            RELEASED.store(true, SeqCst);
            assert_eq!(second, Ok(7));
            assert_eq!(first.join().unwrap(), Ok(7));
        });
    }
}
