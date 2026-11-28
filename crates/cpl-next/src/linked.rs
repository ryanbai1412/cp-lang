//! The build of one program (handbook/design/next/incremental.md, "Whole-program steps"):
//! `program(Job)`, keyed by the root file and what changes the program built from it. Its
//! compute runs the whole pipeline (`compile`) until decls, check and lower answer per item;
//! the `cpl-next` crate move makes it read `generic(ItemId)` and link's per-item answers, and
//! `next_snapshot_keyed_queries` counts it until then.
use crate::facade::Program;
use crate::{compile, Diag, Std};
use cpl_contract::FileId;
use cpl_next_db::db::{Db, DbError, Input, Query};
use cpl_next_db::items::Source;
use cpl_next_decls::modules;
use cpl_next_ir::build::{Checks, Judge};
use std::sync::Arc;

/// What a build is of: the root file, and each setting that changes the program built from it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Job {
    pub root: FileId,
    /// Its runs report the semantic events, for trace.
    pub events: bool,
    /// Every `@fast` fn runs its CP body, not its op.
    pub bodies: bool,
    pub judge: Option<Judge>,
    pub checks: Checks,
}

/// A value the snapshot shares, the same only as the same allocation.
pub struct Shared<T>(pub Arc<T>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<T> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for Shared<T> {}

/// Std's modules by name, the core first: each one's text is the `Source` of its `std_file`.
pub struct StdModules;
impl Input for StdModules {
    type Key = ();
    type Value = Arc<[String]>;
}

/// `std()`: the std the snapshot is checked against, read from its `StdModules`' sources.
pub struct StdLib;
impl Query for StdLib {
    type Key = ();
    type Value = Shared<Std>;
    const NAME: &'static str = "std";
    fn compute(db: &Db, _: &()) -> Result<Shared<Std>, DbError> {
        let names = db.input::<StdModules>(&())?.unwrap_or_default();
        Ok(Shared(Arc::new(Std::from_db(db, &names))))
    }
}

/// Sets `db`'s std to `texts`, by module name, the core first.
pub fn set_std(db: &Db, texts: &[(String, Arc<str>)]) -> Result<(), DbError> {
    for (name, text) in texts {
        db.set::<Source>(modules::std_file(name), text.clone())?;
    }
    db.set::<StdModules>((), texts.iter().map(|(n, _)| n.clone()).collect())
}

/// A built program, or every diagnostic that stops it, in layer order. Two builds are the
/// same answer only as the same allocation, so a rebuilt program counts as changed.
#[derive(Clone)]
pub struct Built(pub Result<Arc<Program>, Arc<[Diag]>>);

impl PartialEq for Built {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Ok(a), Ok(b)) => Arc::ptr_eq(a, b),
            (Err(a), Err(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for Built {}

/// `program(Job)`: the root file built, with the optimized fns its runs use.
pub struct Linked;
impl Query for Linked {
    type Key = Job;
    type Value = Built;
    const NAME: &'static str = "program";
    fn compute(db: &Db, job: &Job) -> Result<Built, DbError> {
        if db.input::<Source>(&job.root)?.is_none() {
            return Ok(Built(Err(Arc::from(Vec::new()))));
        }
        let std = db.get::<StdLib>(&())?;
        let built = compile(
            db,
            &std.0,
            &job.root,
            job.events,
            job.judge.as_ref(),
            job.checks,
        );
        Ok(Built(
            (built.map(|mut c| {
                if job.bodies {
                    c.without_fast();
                }
                Arc::new(Program(c))
            }))
            .map_err(Arc::from),
        ))
    }
}
