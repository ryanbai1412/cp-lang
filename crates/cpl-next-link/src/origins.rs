//! A trap names a site of the user's file by one number for the whole program.
use cpl_next_ir::ids::SourceId;
use cpl_next_ir::prog::Program;
use std::collections::HashMap;

/// Numbers each site lowering marked the user's (`Func::origins` 1) into `Program::origins`,
/// one number per place however many instances hold it, so every engine names it alike.
pub(crate) fn number_origins(p: &mut Program) {
    let mut seen: HashMap<SourceId, u32> = HashMap::new();
    for f in &mut p.insts {
        for (o, at) in f.origins.iter_mut().zip(&f.sites) {
            if *o != 0 {
                *o = *seen.entry(at.clone()).or_insert_with(|| {
                    p.origins.push(at.clone());
                    p.origins.len() as u32
                });
            }
        }
    }
}
