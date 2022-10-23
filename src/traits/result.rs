pub enum SoundError {
    NoError
}

pub type Result<T> = std::result::Result<T, SoundError>;