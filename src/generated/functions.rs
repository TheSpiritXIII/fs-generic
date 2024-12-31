// This file is auto-generated. DO NOT edit by hand. See README.md for more details.
#![allow(clippy::tabs_in_doc_comments)]
use std::fs::Metadata;
use std::fs::Permissions;
use std::fs::ReadDir;
use std::io;
use std::path;

pub trait Fs {
	/// Returns the canonical, absolute form of a path with all intermediate
	/// components normalized and symbolic links resolved.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `realpath` function on Unix
	/// and the `CreateFile` and `GetFinalPathNameByHandle` functions on Windows.
	/// Note that this [may change in the future][changes].
	///
	/// On Windows, this converts the path to use [extended length path][path]
	/// syntax, which allows your program to use longer path names, but means you
	/// can only join backslash-delimited paths to it, and it may be incompatible
	/// with other applications (if passed to the application on the command-line,
	/// or written to a file another application may read).
	///
	/// [changes]: io#platform-specific-behavior
	/// [path]: https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `path` does not exist.
	/// * A non-final component in path is not a directory.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	let path = fs::canonicalize("../a/../foo.txt")?;
	/// 	Ok(())
	/// }
	/// ```
	fn canonicalize<P: AsRef<path::Path>>(path: P) -> io::Result<path::PathBuf>;

	/// Copies the contents of one file to another. This function will also
	/// copy the permission bits of the original file to the destination file.
	///
	/// This function will **overwrite** the contents of `to`.
	///
	/// Note that if `from` and `to` both point to the same file, then the file
	/// will likely get truncated by this operation.
	///
	/// On success, the total number of bytes copied is returned and it is equal to
	/// the length of the `to` file as reported by `metadata`.
	///
	/// If you want to copy the contents of one file to another and you’re
	/// working with [`File`]s, see the [`io::copy`](io::copy()) function.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `open` function in Unix
	/// with `O_RDONLY` for `from` and `O_WRONLY`, `O_CREAT`, and `O_TRUNC` for `to`.
	/// `O_CLOEXEC` is set for returned file descriptors.
	///
	/// On Linux (including Android), this function attempts to use `copy_file_range(2)`,
	/// and falls back to reading and writing if that is not possible.
	///
	/// On Windows, this function currently corresponds to `CopyFileEx`. Alternate
	/// NTFS streams are copied but only the size of the main stream is returned by
	/// this function.
	///
	/// On MacOS, this function corresponds to `fclonefileat` and `fcopyfile`.
	///
	/// Note that platform-specific behavior [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `from` is neither a regular file nor a symlink to a regular file.
	/// * `from` does not exist.
	/// * The current process does not have the permission rights to read `from` or write `to`.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::copy("foo.txt", "bar.txt")?; // Copy foo.txt to bar.txt
	/// 	Ok(())
	/// }
	/// ```
	fn copy<P: AsRef<path::Path>, Q: AsRef<path::Path>>(from: P, to: Q) -> io::Result<u64>;

	/// Creates a new, empty directory at the provided path
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `mkdir` function on Unix
	/// and the `CreateDirectoryW` function on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// **NOTE**: If a parent of the given path doesn't exist, this function will
	/// return an error. To create a directory and all its missing parents at the
	/// same time, use the [`create_dir_all`] function.
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * User lacks permissions to create directory at `path`.
	/// * A parent of the given path doesn't exist. (To create a directory and all its missing
	///   parents at the same time, use the [`create_dir_all`] function.)
	/// * `path` already exists.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::create_dir("/some/dir")?;
	/// 	Ok(())
	/// }
	/// ```
	fn create_dir<P: AsRef<path::Path>>(path: P) -> io::Result<()>;

	/// Recursively create a directory and all of its parent components if they
	/// are missing.
	///
	/// If this function returns an error, some of the parent components might have
	/// been created already.
	///
	/// If the empty path is passed to this function, it always succeeds without
	/// creating any directories.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to multiple calls to the `mkdir`
	/// function on Unix and the `CreateDirectoryW` function on Windows.
	///
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// The function will return an error if any directory specified in path does not exist and
	/// could not be created. There may be other error conditions; see [`fs::create_dir`] for
	/// specifics.
	///
	/// Notable exception is made for situations where any of the directories
	/// specified in the `path` could not be created as it was being created concurrently.
	/// Such cases are considered to be successful. That is, calling `create_dir_all`
	/// concurrently from multiple threads or processes is guaranteed not to fail
	/// due to a race condition with itself.
	///
	/// [`fs::create_dir`]: create_dir
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::create_dir_all("/some/dir")?;
	/// 	Ok(())
	/// }
	/// ```
	fn create_dir_all<P: AsRef<path::Path>>(path: P) -> io::Result<()>;

	/// Returns `Ok(true)` if the path points at an existing entity.
	///
	/// This function will traverse symbolic links to query information about the
	/// destination file. In case of broken symbolic links this will return `Ok(false)`.
	///
	/// As opposed to the [`Path::exists`] method, this will only return `Ok(true)` or `Ok(false)`
	/// if the path was _verified_ to exist or not exist. If its existence can neither be confirmed
	/// nor denied, an `Err(_)` will be propagated instead. This can be the case if e.g. listing
	/// permission is denied on one of the parent directories.
	///
	/// Note that while this avoids some pitfalls of the `exists()` method, it still can not
	/// prevent time-of-check to time-of-use (TOCTOU) bugs. You should only use it in scenarios
	/// where those bugs are not an issue.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// assert!(!fs::exists("does_not_exist.txt")
	/// 	.expect("Can't check existence of file does_not_exist.txt"));
	/// assert!(fs::exists("/root/secret_file.txt").is_err());
	/// ```
	///
	/// [`Path::exists`]: crate::path::Path::exists
	fn exists<P: AsRef<path::Path>>(path: P) -> io::Result<bool>;

	/// Creates a new hard link on the filesystem.
	///
	/// The `link` path will be a link pointing to the `original` path. Note that
	/// systems often require these two paths to both be located on the same
	/// filesystem.
	///
	/// If `original` names a symbolic link, it is platform-specific whether the
	/// symbolic link is followed. On platforms where it's possible to not follow
	/// it, it is not followed, and the created hard link points to the symbolic
	/// link itself.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds the `CreateHardLink` function on Windows.
	/// On most Unix systems, it corresponds to the `linkat` function with no flags.
	/// On Android, VxWorks, and Redox, it instead corresponds to the `link` function.
	/// On MacOS, it uses the `linkat` function if it is available, but on very old
	/// systems where `linkat` is not available, `link` is selected at runtime instead.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * The `original` path is not a file or doesn't exist.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::hard_link("a.txt", "b.txt")?; // Hard link a.txt to b.txt
	/// 	Ok(())
	/// }
	/// ```
	fn hard_link<P: AsRef<path::Path>, Q: AsRef<path::Path>>(
		original: P,
		link: Q,
	) -> io::Result<()>;

	/// Given a path, queries the file system to get information about a file,
	/// directory, etc.
	///
	/// This function will traverse symbolic links to query information about the
	/// destination file.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `stat` function on Unix
	/// and the `GetFileInformationByHandle` function on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * The user lacks permissions to perform `metadata` call on `path`.
	/// * `path` does not exist.
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	let attr = fs::metadata("/some/file/path.txt")?;
	/// 	// inspect attr ...
	/// 	Ok(())
	/// }
	/// ```
	fn metadata<P: AsRef<path::Path>>(path: P) -> io::Result<Metadata>;

	/// Reads the entire contents of a file into a bytes vector.
	///
	/// This is a convenience function for using [`File::open`] and [`read_to_end`]
	/// with fewer imports and without an intermediate variable.
	///
	/// [`read_to_end`]: Read::read_to_end
	///
	/// # Errors
	///
	/// This function will return an error if `path` does not already exist.
	/// Other errors may also be returned according to [`OpenOptions::open`].
	///
	/// While reading from the file, this function handles [`io::ErrorKind::Interrupted`]
	/// with automatic retries. See [io::Read] documentation for details.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
	/// 	let data: Vec<u8> = fs::read("image.jpg")?;
	/// 	assert_eq!(
	/// 		data[0..3],
	/// 		[
	/// 			0xFF,
	/// 			0xD8,
	/// 			0xFF
	/// 		]
	/// 	);
	/// 	Ok(())
	/// }
	/// ```
	fn read<P: AsRef<path::Path>>(path: P) -> io::Result<Vec<u8>>;

	/// Returns an iterator over the entries within a directory.
	///
	/// The iterator will yield instances of <code>[io::Result]<[DirEntry]></code>.
	/// New errors may be encountered after an iterator is initially constructed.
	/// Entries for the current and parent directories (typically `.` and `..`) are
	/// skipped.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `opendir` function on Unix
	/// and the `FindFirstFileEx` function on Windows. Advancing the iterator
	/// currently corresponds to `readdir` on Unix and `FindNextFile` on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// The order in which this iterator returns entries is platform and filesystem
	/// dependent.
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * The provided `path` doesn't exist.
	/// * The process lacks permissions to view the contents.
	/// * The `path` points at a non-directory file.
	///
	/// # Examples
	///
	/// ```
	/// use std::fs::DirEntry;
	/// use std::fs::{self};
	/// use std::io;
	/// use std::path::Path;
	///
	/// // one possible implementation of walking a directory only visiting files
	/// fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
	/// 	if dir.is_dir() {
	/// 		for entry in fs::read_dir(dir)? {
	/// 			let entry = entry?;
	/// 			let path = entry.path();
	/// 			if path.is_dir() {
	/// 				visit_dirs(&path, cb)?;
	/// 			} else {
	/// 				cb(&entry);
	/// 			}
	/// 		}
	/// 	}
	/// 	Ok(())
	/// }
	/// ```
	///
	/// ```rust,no_run
	/// use std::fs;
	/// use std::io;
	///
	/// fn main() -> io::Result<()> {
	/// 	let mut entries = fs::read_dir(".")?
	/// 		.map(|res| res.map(|e| e.path()))
	/// 		.collect::<Result<Vec<_>, io::Error>>()?;
	///
	/// 	// The order in which `read_dir` returns entries is not guaranteed. If reproducible
	/// 	// ordering is required the entries should be explicitly sorted.
	///
	/// 	entries.sort();
	///
	/// 	// The entries have now been sorted by their path.
	///
	/// 	Ok(())
	/// }
	/// ```
	fn read_dir<P: AsRef<path::Path>>(path: P) -> io::Result<ReadDir>;

	/// Reads a symbolic link, returning the file that the link points to.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `readlink` function on Unix
	/// and the `CreateFile` function with `FILE_FLAG_OPEN_REPARSE_POINT` and
	/// `FILE_FLAG_BACKUP_SEMANTICS` flags on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `path` is not a symbolic link.
	/// * `path` does not exist.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	let path = fs::read_link("a.txt")?;
	/// 	Ok(())
	/// }
	/// ```
	fn read_link<P: AsRef<path::Path>>(path: P) -> io::Result<path::PathBuf>;

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
	/// use std::error::Error;
	/// use std::fs;
	///
	/// fn main() -> Result<(), Box<dyn Error>> {
	/// 	let message: String = fs::read_to_string("message.txt")?;
	/// 	println!("{}", message);
	/// 	Ok(())
	/// }
	/// ```
	fn read_to_string<P: AsRef<path::Path>>(path: P) -> io::Result<String>;

	/// Removes an empty directory.
	///
	/// If you want to remove a directory that is not empty, as well as all
	/// of its contents recursively, consider using [`remove_dir_all`]
	/// instead.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `rmdir` function on Unix
	/// and the `RemoveDirectory` function on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `path` doesn't exist.
	/// * `path` isn't a directory.
	/// * The user lacks permissions to remove the directory at the provided `path`.
	/// * The directory isn't empty.
	///
	/// This function will only ever return an error of kind `NotFound` if the given
	/// path does not exist. Note that the inverse is not true,
	/// ie. if a path does not exist, its removal may fail for a number of reasons,
	/// such as insufficient permissions.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::remove_dir("/some/dir")?;
	/// 	Ok(())
	/// }
	/// ```
	fn remove_dir<P: AsRef<path::Path>>(path: P) -> io::Result<()>;

	/// Removes a directory at this path, after removing all its contents. Use
	/// carefully!
	///
	/// This function does **not** follow symbolic links and it will simply remove the
	/// symbolic link itself.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to `openat`, `fdopendir`, `unlinkat` and `lstat`
	/// functions on Unix (except for REDOX) and the `CreateFileW`, `GetFileInformationByHandleEx`,
	/// `SetFileInformationByHandle`, and `NtCreateFile` functions on Windows. Note that, this
	/// [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// On REDOX, as well as when running in Miri for any target, this function is not protected
	/// against time-of-check to time-of-use (TOCTOU) race conditions, and should not be used in
	/// security-sensitive code on those platforms. All other platforms are protected.
	///
	/// # Errors
	///
	/// See [`fs::remove_file`] and [`fs::remove_dir`].
	///
	/// `remove_dir_all` will fail if `remove_dir` or `remove_file` fail on any constituent paths,
	/// including the root `path`. As a result, the directory you are deleting must exist, meaning
	/// that this function is not idempotent. Additionally, `remove_dir_all` will also fail if the
	/// `path` is not a directory.
	///
	/// Consider ignoring the error if validating the removal is not required for your use case.
	///
	/// [`io::ErrorKind::NotFound`] is only returned if no removal occurs.
	///
	/// [`fs::remove_file`]: remove_file
	/// [`fs::remove_dir`]: remove_dir
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::remove_dir_all("/some/dir")?;
	/// 	Ok(())
	/// }
	/// ```
	fn remove_dir_all<P: AsRef<path::Path>>(path: P) -> io::Result<()>;

	/// Removes a file from the filesystem.
	///
	/// Note that there is no
	/// guarantee that the file is immediately deleted (e.g., depending on
	/// platform, other open file descriptors may prevent immediate removal).
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `unlink` function on Unix
	/// and the `DeleteFile` function on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `path` points to a directory.
	/// * The file doesn't exist.
	/// * The user lacks permissions to remove the file.
	///
	/// This function will only ever return an error of kind `NotFound` if the given
	/// path does not exist. Note that the inverse is not true,
	/// ie. if a path does not exist, its removal may fail for a number of reasons,
	/// such as insufficient permissions.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::remove_file("a.txt")?;
	/// 	Ok(())
	/// }
	/// ```
	fn remove_file<P: AsRef<path::Path>>(path: P) -> io::Result<()>;

	/// Renames a file or directory to a new name, replacing the original file if
	/// `to` already exists.
	///
	/// This will not work if the new name is on a different mount point.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `rename` function on Unix
	/// and the `SetFileInformationByHandle` function on Windows.
	///
	/// Because of this, the behavior when both `from` and `to` exist differs. On
	/// Unix, if `from` is a directory, `to` must also be an (empty) directory. If
	/// `from` is not a directory, `to` must also be not a directory. The behavior
	/// on Windows is the same on Windows 10 1607 and higher if `FileRenameInfoEx`
	/// is supported by the filesystem; otherwise, `from` can be anything, but
	/// `to` must *not* be a directory.
	///
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `from` does not exist.
	/// * The user lacks permissions to view contents.
	/// * `from` and `to` are on separate filesystems.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::rename("a.txt", "b.txt")?; // Rename a.txt to b.txt
	/// 	Ok(())
	/// }
	/// ```
	fn rename<P: AsRef<path::Path>, Q: AsRef<path::Path>>(from: P, to: Q) -> io::Result<()>;

	/// Changes the permissions found on a file or a directory.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `chmod` function on Unix
	/// and the `SetFileAttributes` function on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * `path` does not exist.
	/// * The user lacks the permission to change attributes of the file.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	let mut perms = fs::metadata("foo.txt")?.permissions();
	/// 	perms.set_readonly(true);
	/// 	fs::set_permissions("foo.txt", perms)?;
	/// 	Ok(())
	/// }
	/// ```
	fn set_permissions<P: AsRef<path::Path>>(path: P, perm: Permissions) -> io::Result<()>;

	/// Queries the metadata about a file without following symlinks.
	///
	/// # Platform-specific behavior
	///
	/// This function currently corresponds to the `lstat` function on Unix
	/// and the `GetFileInformationByHandle` function on Windows.
	/// Note that, this [may change in the future][changes].
	///
	/// [changes]: io#platform-specific-behavior
	///
	/// # Errors
	///
	/// This function will return an error in the following situations, but is not
	/// limited to just these cases:
	///
	/// * The user lacks permissions to perform `metadata` call on `path`.
	/// * `path` does not exist.
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	let attr = fs::symlink_metadata("/some/file/path.txt")?;
	/// 	// inspect attr ...
	/// 	Ok(())
	/// }
	/// ```
	fn symlink_metadata<P: AsRef<path::Path>>(path: P) -> io::Result<Metadata>;

	/// Writes a slice as the entire contents of a file.
	///
	/// This function will create a file if it does not exist,
	/// and will entirely replace its contents if it does.
	///
	/// Depending on the platform, this function may fail if the
	/// full directory path does not exist.
	///
	/// This is a convenience function for using [`File::create`] and [`write_all`]
	/// with fewer imports.
	///
	/// [`write_all`]: Write::write_all
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::fs;
	///
	/// fn main() -> std::io::Result<()> {
	/// 	fs::write("foo.txt", b"Lorem ipsum")?;
	/// 	fs::write("bar.txt", "dolor sit")?;
	/// 	Ok(())
	/// }
	/// ```
	fn write<P: AsRef<path::Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()>;
}

pub struct Native {}

impl Fs for Native {
	fn canonicalize<P: AsRef<path::Path>>(path: P) -> io::Result<path::PathBuf> {
		std::fs::canonicalize(path)
	}

	fn copy<P: AsRef<path::Path>, Q: AsRef<path::Path>>(from: P, to: Q) -> io::Result<u64> {
		std::fs::copy(from, to)
	}

	fn create_dir<P: AsRef<path::Path>>(path: P) -> io::Result<()> {
		std::fs::create_dir(path)
	}

	fn create_dir_all<P: AsRef<path::Path>>(path: P) -> io::Result<()> {
		std::fs::create_dir_all(path)
	}

	fn exists<P: AsRef<path::Path>>(path: P) -> io::Result<bool> {
		std::fs::exists(path)
	}

	fn hard_link<P: AsRef<path::Path>, Q: AsRef<path::Path>>(
		original: P,
		link: Q,
	) -> io::Result<()> {
		std::fs::hard_link(original, link)
	}

	fn metadata<P: AsRef<path::Path>>(path: P) -> io::Result<Metadata> {
		std::fs::metadata(path)
	}

	fn read<P: AsRef<path::Path>>(path: P) -> io::Result<Vec<u8>> {
		std::fs::read(path)
	}

	fn read_dir<P: AsRef<path::Path>>(path: P) -> io::Result<ReadDir> {
		std::fs::read_dir(path)
	}

	fn read_link<P: AsRef<path::Path>>(path: P) -> io::Result<path::PathBuf> {
		std::fs::read_link(path)
	}

	fn read_to_string<P: AsRef<path::Path>>(path: P) -> io::Result<String> {
		std::fs::read_to_string(path)
	}

	fn remove_dir<P: AsRef<path::Path>>(path: P) -> io::Result<()> {
		std::fs::remove_dir(path)
	}

	fn remove_dir_all<P: AsRef<path::Path>>(path: P) -> io::Result<()> {
		std::fs::remove_dir_all(path)
	}

	fn remove_file<P: AsRef<path::Path>>(path: P) -> io::Result<()> {
		std::fs::remove_file(path)
	}

	fn rename<P: AsRef<path::Path>, Q: AsRef<path::Path>>(from: P, to: Q) -> io::Result<()> {
		std::fs::rename(from, to)
	}

	fn set_permissions<P: AsRef<path::Path>>(path: P, perm: Permissions) -> io::Result<()> {
		std::fs::set_permissions(path, perm)
	}

	fn symlink_metadata<P: AsRef<path::Path>>(path: P) -> io::Result<Metadata> {
		std::fs::symlink_metadata(path)
	}

	fn write<P: AsRef<path::Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
		std::fs::write(path, contents)
	}
}
