use anyhow::Result;
use ort::session::{Session, builder::GraphOptimizationLevel};
use std::path::Path;

pub fn load_session<P: AsRef<Path>>(model_path: P) -> Result<Session> {
    let sess = Session::builder()?
        .with_inter_threads(6)?
        .with_optimization_level(GraphOptimizationLevel::Level1)?
        .commit_from_file(model_path)?;
    Ok(sess)
}
