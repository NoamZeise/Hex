#[derive(Debug)]
pub enum TiledError {
    FileReadError(String, String),
    ParseError(String),
    ParseBytesError(),
    UnsupportedType(),
    MissingPoint(),
}
