//! A shared box the VM gets back when its last holder drops it, so a loop
//! that makes and drops a struct or a view each round reuses one box
//! instead of asking the allocator every round.

use super::{Cells, ListViewVal, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::LocalKey;

/// Boxes kept per type; a run holding more live at once frees the rest.
const KEPT: usize = 256;

type Pool<T> = RefCell<Vec<Rc<T>>>;

pub(crate) trait Recycle: Sized + 'static {
    /// The boxes waiting to be reused.
    fn pool() -> &'static LocalKey<Pool<Self>>;
    /// A box standing in for one being taken back.
    fn blank() -> &'static LocalKey<Rc<Self>>;
    /// Drops what the value holds, keeping its storage.
    fn reset(&mut self);
}

impl Recycle for Vec<Value> {
    fn pool() -> &'static LocalKey<Pool<Self>> {
        thread_local!(static POOL: Pool<Vec<Value>> = const { RefCell::new(Vec::new()) });
        &POOL
    }
    fn blank() -> &'static LocalKey<Rc<Self>> {
        thread_local!(static BLANK: Rc<Vec<Value>> = Rc::new(Vec::new()));
        &BLANK
    }
    fn reset(&mut self) {
        self.clear();
    }
}

thread_local!(static NO_CELLS: Rc<Cells> = Rc::new(Cells::default()));

impl Recycle for ListViewVal {
    fn pool() -> &'static LocalKey<Pool<Self>> {
        thread_local!(static POOL: Pool<ListViewVal> = const { RefCell::new(Vec::new()) });
        &POOL
    }
    fn blank() -> &'static LocalKey<Rc<Self>> {
        thread_local!(static BLANK: Rc<ListViewVal> =
            Rc::new(ListViewVal::whole(NO_CELLS.with(Rc::clone), false)));
        &BLANK
    }
    fn reset(&mut self) {
        if let Ok(none) = NO_CELLS.try_with(Rc::clone) {
            self.base = none;
        }
    }
}

/// An `Rc<T>` whose box goes back to `T`'s pool when the last holder drops it.
#[derive(Debug)]
pub(crate) struct Shared<T: Recycle>(Rc<T>);

impl<T: Recycle> Shared<T> {
    fn reused() -> Option<Rc<T>> {
        T::pool()
            .try_with(|p| p.try_borrow_mut().ok()?.pop())
            .ok()
            .flatten()
    }

    pub fn new(value: T) -> Shared<T> {
        if let Some(mut rc) = Self::reused() {
            if let Some(slot) = Rc::get_mut(&mut rc) {
                *slot = value;
                return Shared(rc);
            }
        }
        Shared(Rc::new(value))
    }

    pub fn make_mut(this: &mut Shared<T>) -> &mut T
    where
        T: Clone,
    {
        Rc::make_mut(&mut this.0)
    }
}

impl Shared<Vec<Value>> {
    /// The fields `field(0..n)`, in a reused box when one is free.
    pub fn collect(n: usize, mut field: impl FnMut(usize) -> Value) -> Shared<Vec<Value>> {
        if let Some(mut rc) = Self::reused() {
            if let Some(fs) = Rc::get_mut(&mut rc) {
                fs.extend((0..n).map(&mut field));
                return Shared(rc);
            }
        }
        Shared(Rc::new((0..n).map(field).collect()))
    }
}

impl<T: Recycle> Clone for Shared<T> {
    fn clone(&self) -> Shared<T> {
        Shared(Rc::clone(&self.0))
    }
}

impl<T: Recycle> std::ops::Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Recycle> Drop for Shared<T> {
    fn drop(&mut self) {
        if Rc::strong_count(&self.0) != 1 || Rc::weak_count(&self.0) != 0 {
            return;
        }
        let Ok(blank) = T::blank().try_with(Rc::clone) else {
            return;
        };
        let mut rc = std::mem::replace(&mut self.0, blank);
        if let Some(value) = Rc::get_mut(&mut rc) {
            // Outside the pool's borrow: what the value held may hold boxes too.
            value.reset();
        }
        let _ = T::pool().try_with(|p| {
            if let Ok(mut p) = p.try_borrow_mut() {
                if p.len() < KEPT {
                    p.push(rc);
                }
            }
        });
    }
}
