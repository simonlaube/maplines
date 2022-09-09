#[derive(Debug)]
pub enum MaplineError {
    GpxFileNotValid,
    FitFileNotAnActivity,
    ImportError(String),
    TrackAlreadyImported,
    CouldNotLoadElevation,
}