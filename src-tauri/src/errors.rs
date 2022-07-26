#[derive(Error, Debug)]
pub enum ImportError {
    GpxFileNotValid("The given GPX file is not valid."),
}