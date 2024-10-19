// This file is auto-generated. DO NOT edit by hand. See README.md for more details.
#![allow(clippy::tabs_in_doc_comments)]

/// A builder used to create directories in various manners.
///
/// This builder also supports platform-specific options.
pub trait DirBuilder {
	// fn new() -> DirBuilder;
	// fn recursive(self: &mut Self, recursive: bool, ) -> &mut Self;
	// fn create<P: AsRef<Path,> + , >(self: &Self, path: P, ) -> io::Result<(),>;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl core::fmt::Debug
}

// impl DirBuilder for std::fs::DirBuilder {}

/// Entries returned by the [`ReadDir`] iterator.
///
/// An instance of `DirEntry` represents an entry inside of a directory on the
/// filesystem. Each entry can be inspected via methods to learn about the full
/// path or possibly other metadata through per-platform extension traits.
///
/// # Platform-specific behavior
///
/// On Unix, the `DirEntry` struct contains an internal reference to the open
/// directory. Holding `DirEntry` objects will consume a file handle even
/// after the `ReadDir` iterator is dropped.
///
/// Note that this [may change in the future][changes].
///
/// [changes]: io#platform-specific-behavior
pub trait DirEntry {
	// fn path(self: &Self, ) -> PathBuf;
	// fn metadata(self: &Self, ) -> io::Result<Metadata,>;
	// fn file_type(self: &Self, ) -> io::Result<FileType,>;
	// fn file_name(self: &Self, ) -> OsString;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl core::fmt::Debug
	// impl std::os::wasi::fs::DirEntryExt
}

// impl DirEntry for std::fs::DirEntry {}

/// An object providing access to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with. Files also implement [`Seek`] to alter the logical cursor
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope.  Errors detected
/// on closing are ignored by the implementation of `Drop`.  Use the method
/// [`sync_all`] if these errors must be manually handled.
///
/// `File` does not buffer reads and writes. For efficiency, consider wrapping the
/// file in a [`BufReader`] or [`BufWriter`] when performing many small [`read`]
/// or [`write`] calls, unless unbuffered reads and writes are required.
///
/// # Examples
///
/// Creates a new file and write bytes to it (you can also use [`write`]):
///
/// ```no_run
/// use std::fs::File;
/// use std::io::prelude::*;
///
/// fn main() -> std::io::Result<()> {
/// 	let mut file = File::create("foo.txt")?;
/// 	file.write_all(b"Hello, world!")?;
/// 	Ok(())
/// }
/// ```
///
/// Reads the contents of a file into a [`String`] (you can also use [`read`]):
///
/// ```no_run
/// use std::fs::File;
/// use std::io::prelude::*;
///
/// fn main() -> std::io::Result<()> {
/// 	let mut file = File::open("foo.txt")?;
/// 	let mut contents = String::new();
/// 	file.read_to_string(&mut contents)?;
/// 	assert_eq!(contents, "Hello, world!");
/// 	Ok(())
/// }
/// ```
///
/// Using a buffered [`Read`]er:
///
/// ```no_run
/// use std::fs::File;
/// use std::io::prelude::*;
/// use std::io::BufReader;
///
/// fn main() -> std::io::Result<()> {
/// 	let file = File::open("foo.txt")?;
/// 	let mut buf_reader = BufReader::new(file);
/// 	let mut contents = String::new();
/// 	buf_reader.read_to_string(&mut contents)?;
/// 	assert_eq!(contents, "Hello, world!");
/// 	Ok(())
/// }
/// ```
///
/// Note that, although read and write methods require a `&mut File`, because
/// of the interfaces for [`Read`] and [`Write`], the holder of a `&File` can
/// still modify the file, either through methods that take `&File` or by
/// retrieving the underlying OS object and modifying the file that way.
/// Additionally, many operating systems allow concurrent modification of files
/// by different processes. Avoid assuming that holding a `&File` means that the
/// file will not change.
///
/// # Platform-specific behavior
///
/// On Windows, the implementation of [`Read`] and [`Write`] traits for `File`
/// perform synchronous I/O operations. Therefore the underlying file must not
/// have been opened for asynchronous I/O (e.g. by using `FILE_FLAG_OVERLAPPED`).
///
/// [`BufReader`]: io::BufReader
/// [`BufWriter`]: io::BufWriter
/// [`sync_all`]: File::sync_all
/// [`write`]: File::write
/// [`read`]: File::read
pub trait File {
	// fn open<P: AsRef<Path,> + , >(path: P, ) -> io::Result<File,>;
	// fn open_buffered<P: AsRef<Path,> + , >(path: P, ) -> io::Result<io::BufReader<File,>,>;
	// fn create<P: AsRef<Path,> + , >(path: P, ) -> io::Result<File,>;
	// fn create_buffered<P: AsRef<Path,> + , >(path: P, ) -> io::Result<io::BufWriter<File,>,>;
	// fn create_new<P: AsRef<Path,> + , >(path: P, ) -> io::Result<File,>;
	// fn options() -> OpenOptions;
	// fn sync_all(self: &Self, ) -> io::Result<(),>;
	// fn sync_data(self: &Self, ) -> io::Result<(),>;
	// fn set_len(self: &Self, size: u64, ) -> io::Result<(),>;
	// fn metadata(self: &Self, ) -> io::Result<Metadata,>;
	// fn try_clone(self: &Self, ) -> io::Result<File,>;
	// fn set_permissions(self: &Self, perm: Permissions, ) -> io::Result<(),>;
	// fn set_times(self: &Self, times: FileTimes, ) -> io::Result<(),>;
	// fn set_modified(self: &Self, time: SystemTime, ) -> io::Result<(),>;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl core::fmt::Debug
	// impl std::io::Read
	// impl std::io::Write
	// impl std::io::Seek
	// impl std::io::Read
	// impl std::io::Write
	// impl std::io::Seek
	// impl IsTerminal
	// impl std::os::wasi::fs::FileExt
	// impl AsHandle
	// impl core::convert::From<File,>
	// impl core::convert::From<OwnedHandle,>
	// impl AsRawHandle
	// impl FromRawHandle
	// impl IntoRawHandle
	// impl AsRawFd
	// impl FromRawFd
	// impl IntoRawFd
	// impl AsFd
	// impl core::convert::From<File,>
	// impl core::convert::From<OwnedFd,>
	// impl core::convert::From<File,>
}

// impl File for std::fs::File {}

/// Representation of the various timestamps on a file.
pub trait FileTimes {
	// fn new() -> Self;
	// fn set_accessed(self: Self, t: SystemTime, ) -> Self;
	// fn set_modified(self: Self, t: SystemTime, ) -> Self;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::clone::CloneToUninit
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl alloc::borrow::ToOwned
	// impl core::marker::Copy
	// impl core::clone::Clone
	// impl core::fmt::Debug
	// impl core::default::Default
}

// impl FileTimes for std::fs::FileTimes {}

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
pub trait FileType {
	// fn is_dir(self: &Self, ) -> bool;
	// fn is_file(self: &Self, ) -> bool;
	// fn is_symlink(self: &Self, ) -> bool;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::clone::CloneToUninit
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl alloc::borrow::ToOwned
	// impl core::marker::Copy
	// impl core::clone::Clone
	// impl core::marker::StructuralPartialEq
	// impl core::cmp::PartialEq
	// impl core::cmp::Eq
	// impl core::hash::Hash
	// impl core::fmt::Debug
	// impl std::os::wasi::fs::FileTypeExt
}

// impl FileType for std::fs::FileType {}

/// Metadata information about a file.
///
/// This structure is returned from the [`metadata`] or
/// [`symlink_metadata`] function or method and represents known
/// metadata about a file such as its permissions, size, modification
/// times, etc.
pub trait Metadata {
	// fn file_type(self: &Self, ) -> FileType;
	// fn is_dir(self: &Self, ) -> bool;
	// fn is_file(self: &Self, ) -> bool;
	// fn is_symlink(self: &Self, ) -> bool;
	// fn len(self: &Self, ) -> u64;
	// fn permissions(self: &Self, ) -> Permissions;
	// fn modified(self: &Self, ) -> io::Result<SystemTime,>;
	// fn accessed(self: &Self, ) -> io::Result<SystemTime,>;
	// fn created(self: &Self, ) -> io::Result<SystemTime,>;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::clone::CloneToUninit
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl alloc::borrow::ToOwned
	// impl core::clone::Clone
	// impl core::fmt::Debug
	// impl std::os::linux::fs::MetadataExt
	// impl std::os::wasi::fs::MetadataExt
}

// impl Metadata for std::fs::Metadata {}

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a [`File`] is opened and
/// what operations are permitted on the open file. The [`File::open`] and
/// [`File::create`] methods are aliases for commonly used options using this
/// builder.
///
/// Generally speaking, when using `OpenOptions`, you'll first call
/// [`OpenOptions::new`], then chain calls to methods to set each option, then
/// call [`OpenOptions::open`], passing the path of the file you're trying to
/// open. This will give you a [`io::Result`] with a [`File`] inside that you
/// can further operate on.
///
/// # Examples
///
/// Opening a file to read:
///
/// ```no_run
/// use std::fs::OpenOptions;
///
/// let file = OpenOptions::new().read(true).open("foo.txt");
/// ```
///
/// Opening a file for both reading and writing, as well as creating it if it
/// doesn't exist:
///
/// ```no_run
/// use std::fs::OpenOptions;
///
/// let file = OpenOptions::new().read(true).write(true).create(true).open("foo.txt");
/// ```
pub trait OpenOptions {
	// fn new() -> Self;
	// fn read(self: &mut Self, read: bool, ) -> &mut Self;
	// fn write(self: &mut Self, write: bool, ) -> &mut Self;
	// fn append(self: &mut Self, append: bool, ) -> &mut Self;
	// fn truncate(self: &mut Self, truncate: bool, ) -> &mut Self;
	// fn create(self: &mut Self, create: bool, ) -> &mut Self;
	// fn create_new(self: &mut Self, create_new: bool, ) -> &mut Self;
	// fn open<P: AsRef<Path,> + , >(self: &Self, path: P, ) -> io::Result<File,>;
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::clone::CloneToUninit
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl alloc::borrow::ToOwned
	// impl core::clone::Clone
	// impl core::fmt::Debug
	// impl std::os::wasi::fs::OpenOptionsExt
}

// impl OpenOptions for std::fs::OpenOptions {}

/// Representation of the various permissions on a file.
///
/// This module only currently provides one bit of information,
/// [`Permissions::readonly`], which is exposed on all currently supported
/// platforms. Unix-specific functionality, such as mode bits, is available
/// through the [`PermissionsExt`] trait.
///
/// [`PermissionsExt`]: crate::os::unix::fs::PermissionsExt
pub trait Permissions {
	// fn readonly(self: &Self, ) -> bool;
	// fn set_readonly(self: &mut Self, readonly: bool, );
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::clone::CloneToUninit
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl alloc::borrow::ToOwned
	// impl core::clone::Clone
	// impl core::marker::StructuralPartialEq
	// impl core::cmp::PartialEq
	// impl core::cmp::Eq
	// impl core::fmt::Debug
}

// impl Permissions for std::fs::Permissions {}

/// Iterator over the entries in a directory.
///
/// This iterator is returned from the [`read_dir`] function of this module and
/// will yield instances of <code>[io::Result]<[DirEntry]></code>. Through a [`DirEntry`]
/// information like the entry's path and possibly other metadata can be
/// learned.
///
/// The order in which this iterator returns entries is platform and filesystem
/// dependent.
///
/// # Errors
///
/// This [`io::Result`] will be an [`Err`] if there's some sort of intermittent
/// IO error during iteration.
pub trait ReadDir {
	// impl core::marker::Send
	// impl core::marker::Sync
	// impl core::marker::Freeze
	// impl core::marker::Unpin
	// impl core::panic::unwind_safe::UnwindSafe
	// impl core::panic::unwind_safe::RefUnwindSafe
	// impl core::borrow::Borrow<T,>
	// impl core::borrow::BorrowMut<T,>
	// impl core::convert::Into<U,>
	// impl core::convert::From<T,>
	// impl core::convert::TryInto<U,>
	// impl core::convert::TryFrom<U,>
	// impl core::any::Any
	// impl core::iter::traits::collect::IntoIterator
	// impl core::fmt::Debug
	// impl core::iter::traits::iterator::Iterator
}

// impl ReadDir for std::fs::ReadDir {}
