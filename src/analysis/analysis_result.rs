use failure::Error;
use failure::Fail;
use std::fmt;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Debug for AnalysisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalysisInfo",)
    }
}

impl fmt::Debug for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalysisError",)
    }
}

#[derive(Fail)]
pub enum AnalysisError {
    #[fail(display = "Analysis timeout")]
    TimeOut,
    #[fail(display = "The fixed-point algorithm reached the maximum iteration, abort")]
    MaxIteration,
}

pub struct AnalysisInfo {
    pub analysis_time: Duration,
}
