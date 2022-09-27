#[derive(Debug)]
pub enum MaplineError {
    FitFileNotAnActivity,
    ImportError(String),
    TrackAlreadyImported,
    CouldNotLoadElevation,
}