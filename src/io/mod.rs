pub mod input;
pub mod output;

mod read;
mod write;
mod result;

pub use read::SoundRead;
pub use write::SoundWrite;
pub use result::Result;