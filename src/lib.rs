use std::{io, path::Path};

/// Reads the entire contents of a file into a string.
///
/// This is a convenience function for using [`File::open`] and [`read_to_string`]
/// with fewer imports and without an intermediate variable.
///
/// [`read_to_string`]: Read::read_to_string
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`].
///
/// If the contents of the file are not valid UTF-8, then an error will also be
/// returned.
///
/// While reading from the file, this function handles [`io::ErrorKind::Interrupted`]
/// with automatic retries. See [io::Read] documentation for details.
///
/// # Examples
///
/// ```no_run
/// use std::fs;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let message: String = fs::read_to_string("message.txt")?;
///     println!("{}", message);
///     Ok(())
/// }
/// ```
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    std::fs::read_to_string(path)
}
