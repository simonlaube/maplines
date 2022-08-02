#[derive(Debug)]
pub enum ImportError {
    GpxFileNotValid,
    FitFileNotAnActivity,
    ImportError(String),
    TrackAlreadyImported,
}