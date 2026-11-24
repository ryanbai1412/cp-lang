//! The test cases of a `@!multitest` program, run once what is above
//! `@!multitest` has run, from the start or from a checkpoint inside it.

use crate::{observe::Observer, trap, value::Value, Interp, Res};

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    /// The test cases, once what is above `@!multitest` has run.
    pub(crate) fn run_after_setup(&mut self) -> Res<()> {
        let program = self.program;
        if let Some(tracer) = &mut self.tracer {
            tracer.divider();
        }
        let left = if program.until_eof {
            None
        } else {
            match self.input.int() {
                // The judge's side of an interactive problem tells the
                // solution how many test cases there are.
                Ok((count, _)) if program.announce_tests => {
                    self.out.extend_from_slice(format!("{count}\n").as_bytes());
                    self.write_out(true);
                    Some(count)
                }
                Ok((count, _)) => Some(count),
                Err(error) => {
                    return trap(
                        program.multitest_span,
                        format!("reading the number of tests: {}", error.message),
                    )
                }
            }
        };
        self.run_cases(left, 0)
    }

    /// The test cases after the first `case`, while `left` (or the input) has more.
    pub(crate) fn run_cases(&mut self, mut left: Option<i64>, mut case: u64) -> Res<()> {
        let program = self.program;
        loop {
            match &mut left {
                Some(left) if *left <= 0 => break,
                Some(left) => *left -= 1,
                None if self.input.at_eof() => break,
                None => {}
            }
            case += 1;
            self.ckpt.cases_left = left;
            self.current_case = Some(case);
            // `ask(.., limit: K)` allows `K` queries in each test case.
            self.queries = 0;
            if let Some(tracer) = &mut self.tracer {
                tracer.start_case(case);
            }
            self.stack.clear();
            self.stack.resize(program.main_locals.len(), Value::Unit);
            if let Some(compiled) = self.vm.clone() {
                self.run_main_vm(&compiled)?;
                continue;
            }
            self.block(&program.main)?;
        }
        Ok(())
    }
}
