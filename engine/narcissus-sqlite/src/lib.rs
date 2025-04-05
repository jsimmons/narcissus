use core::ffi::c_int;
use std::{
    ffi::{CStr, CString, c_char, c_void},
    marker::PhantomData,
    num::NonZeroI32,
    path::Path,
    ptr::NonNull,
};

use std::sync::OnceLock;

use sqlite_sys as ffi;

static SQLITE_GLOBAL_INIT: OnceLock<()> = OnceLock::new();

#[cold]
unsafe fn initialize() {
    unsafe {
        let ret = ffi::sqlite3_initialize();
        if ret != sqlite_sys::SQLITE_OK {
            panic!("error initializing sqlite: {:?}", Error::new(ret));
        }

        #[cfg(debug_assertions)]
        {
            extern "C" fn log(_user: *mut c_void, _result: c_int, msg: *const c_char) {
                let msg = unsafe { CStr::from_ptr(msg) };
                let msg = msg.to_string_lossy();
                println!("sqlite3: {}", msg);
            }

            let ret = ffi::sqlite3_config(
                ffi::SQLITE_CONFIG_LOG,
                log as extern "C" fn(*mut c_void, i32, *const i8),
                std::ptr::null_mut::<c_void>(),
            );
            if ret != sqlite_sys::SQLITE_OK {
                panic!("error installing sqlite logger: {:?}", Error::new(ret));
            }
        }
    }
}

fn check_initalized() {
    SQLITE_GLOBAL_INIT.get_or_init(|| unsafe { initialize() });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCode {
    /// Generic error
    Generic,
    /// Internal logic error in SQLite
    InternalLogic,
    /// Access permission denied
    PermissionDenied,
    /// Callback routine requested an abort
    OperationAborted,
    /// The database file is locked
    DatabaseBusy,
    /// A table in the database is locked
    DatabaseLocked,
    /// A malloc() failed
    OutOfMemory,
    /// Attempt to write a readonly database
    ReadOnly,
    /// Operation terminated by sqlite3_interrupt()*/
    OperationInterrupted,
    /// Some kind of disk I/O error occurred
    SystemIoError,
    /// The database disk image is malformed
    DatabaseCorrupt,
    /// Unknown opcode in sqlite3_file_control()
    NotFound,
    /// Insertion failed because database is full
    DiskFull,
    /// Unable to open the database file
    CannotOpen,
    /// Database lock protocol error
    FileLockingProtocolFailed,
    /// The database schema changed
    SchemaChanged,
    /// String or BLOB exceeds size limit
    TooBig,
    /// Abort due to constraint violation
    ConstraintViolation,
    /// Data type mismatch
    TypeMismatch,
    /// Uses OS features not supported on host
    NoLargeFileSupport,
    /// Authorization denied
    AuthorizationForStatementDenied,
    /// 2nd parameter to sqlite3_bind out of range
    ParameterOutOfRange,
    /// File opened that is not a database file
    NotADatabase,
    /// An execute call yielded results that were ignored
    ExecuteReturnedResults,
    /// A call receiving sql did not consume the entire input string
    TrailingSql,
}

pub struct Error {
    error: ErrorCode,
    extended: c_int,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("error", &self.error)
            .field("extended", &self.extended)
            .finish()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl Error {
    fn execute_returned_results() -> Self {
        Self {
            error: ErrorCode::ExecuteReturnedResults,
            extended: 0,
        }
    }

    fn trailing_sql() -> Self {
        Self {
            error: ErrorCode::TrailingSql,
            extended: 0,
        }
    }

    #[must_use]
    fn new(return_code: c_int) -> Self {
        let error = match return_code & 0xff {
            ffi::SQLITE_INTERNAL => ErrorCode::InternalLogic,
            ffi::SQLITE_PERM => ErrorCode::PermissionDenied,
            ffi::SQLITE_ABORT => ErrorCode::OperationAborted,
            ffi::SQLITE_BUSY => ErrorCode::DatabaseBusy,
            ffi::SQLITE_LOCKED => ErrorCode::DatabaseLocked,
            ffi::SQLITE_NOMEM => ErrorCode::OutOfMemory,
            ffi::SQLITE_READONLY => ErrorCode::ReadOnly,
            ffi::SQLITE_INTERRUPT => ErrorCode::OperationInterrupted,
            ffi::SQLITE_IOERR => ErrorCode::SystemIoError,
            ffi::SQLITE_CORRUPT => ErrorCode::DatabaseCorrupt,
            ffi::SQLITE_NOTFOUND => ErrorCode::NotFound,
            ffi::SQLITE_FULL => ErrorCode::DiskFull,
            ffi::SQLITE_CANTOPEN => ErrorCode::CannotOpen,
            ffi::SQLITE_PROTOCOL => ErrorCode::FileLockingProtocolFailed,
            ffi::SQLITE_SCHEMA => ErrorCode::SchemaChanged,
            ffi::SQLITE_TOOBIG => ErrorCode::TooBig,
            ffi::SQLITE_CONSTRAINT => ErrorCode::ConstraintViolation,
            ffi::SQLITE_MISMATCH => ErrorCode::TypeMismatch,
            ffi::SQLITE_NOLFS => ErrorCode::NoLargeFileSupport,
            ffi::SQLITE_AUTH => ErrorCode::AuthorizationForStatementDenied,
            ffi::SQLITE_RANGE => ErrorCode::ParameterOutOfRange,
            ffi::SQLITE_NOTADB => ErrorCode::NotADatabase,
            ffi::SQLITE_MISUSE => panic!("sqlite misuse"),
            _ => ErrorCode::Generic,
        };

        Error {
            error,
            extended: return_code,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Connection {
    connection: NonNull<ffi::Connection>,
}

impl Connection {
    fn open(filename: &CStr) -> Result<Self> {
        check_initalized();

        let connection = unsafe {
            let mut connection = std::ptr::null_mut();
            let ret = ffi::sqlite3_open_v2(
                filename.as_ptr(),
                &mut connection,
                ffi::SQLITE_OPEN_READWRITE,
                std::ptr::null_mut(),
            );
            if ret != sqlite_sys::SQLITE_OK {
                return Err(Error::new(ret));
            }
            NonNull::new(connection).unwrap()
        };

        Ok(Self { connection })
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_str().expect("path is not valid utf-8");
        let path = CString::new(path).expect("path contained an internal null byte");
        Self::open(path.as_c_str())
    }

    pub fn open_memory() -> Result<Self> {
        Self::open(c":memory:")
    }

    pub fn prepare<'conn>(&'conn self, sql: &str) -> Result<Statement<'conn>> {
        let (statement, trailing) = self.prepare_partial(sql)?;
        if !trailing.is_empty() {
            return Err(Error::trailing_sql());
        }
        Ok(statement)
    }

    pub fn prepare_partial<'conn, 'a>(
        &'conn self,
        sql: &'a str,
    ) -> Result<(Statement<'conn>, &'a str)> {
        let (statement, tail) = unsafe {
            let ptr = sql.as_ptr() as *const c_char;
            // SAFETY: Check len fits into an i32.
            let len = sql.len().try_into().unwrap();

            let mut statement = std::ptr::null_mut();
            let mut tail = std::ptr::null();

            // SAFETY: `ptr` points to utf-8, and sqlite won't read more than len bytes from
            // the pointer.
            let ret = ffi::sqlite3_prepare_v2(
                self.connection.as_ptr(),
                ptr,
                len,
                &mut statement,
                &mut tail,
            );

            if ret != sqlite_sys::SQLITE_OK {
                let err = Error::new(ret);
                return Err(err);
            }

            // SAFETY: `tail` is a pointer to the first byte past the end of first
            // statement in str. So it's always valid to calculate the offset from ptr, and
            // it's also valid to derive a slice from it with the same lifetime as `sql`.
            let consumed_len = tail.offset_from(ptr) as usize;
            let tail_len = sql.len() - consumed_len;
            let tail_slice = std::slice::from_raw_parts(tail as *const u8, tail_len);
            (statement, std::str::from_utf8_unchecked(tail_slice))
        };

        Ok((
            Statement {
                statement,
                marker: PhantomData,
            },
            tail,
        ))
    }

    pub fn execute(&self, sql: &str) -> Result<()> {
        let mut statement = self.prepare(sql)?;
        statement.query().execute()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.connection.as_ptr()) };
    }
}

pub struct Query<'a> {
    statement: &'a Statement<'a>,
}

impl Drop for Query<'_> {
    fn drop(&mut self) {
        let _ = unsafe { ffi::sqlite3_clear_bindings(self.statement.statement) };
        let _ = unsafe { ffi::sqlite3_reset(self.statement.statement) };
    }
}

impl<'a> Query<'a> {
    pub fn bind_i32(self, index: i32, value: i32) -> Self {
        unsafe { ffi::sqlite3_bind_int(self.statement.statement, index, value) }
        self
    }

    pub fn bind_i64(self, index: i32, value: i64) -> Self {
        unsafe { ffi::sqlite3_bind_int64(self.statement.statement, index, value) }
        self
    }

    pub fn bind_f64(self, index: i32, value: f64) -> Self {
        unsafe { ffi::sqlite3_bind_double(self.statement.statement, index, value) }
        self
    }

    pub fn bind_text(self, index: i32, text: &'a str) -> Query<'a> {
        unsafe {
            ffi::sqlite3_bind_text64(
                self.statement.statement,
                index,
                text.as_ptr() as *const i8,
                text.len() as u64,
                ffi::SQLITE_STATIC,
                ffi::SQLITE_UTF8,
            )
        }
        self
    }

    pub fn bind_blob(self, index: i32, data: &'a [u8]) -> Query<'a> {
        unsafe {
            ffi::sqlite3_bind_blob64(
                self.statement.statement,
                index,
                data.as_ptr() as *mut c_void,
                data.len() as u64,
                ffi::SQLITE_STATIC,
            )
        }
        self
    }

    pub fn bind_null(self, index: i32) -> Self {
        unsafe { ffi::sqlite3_bind_null(self.statement.statement, index) }
        self
    }

    pub fn execute(self) -> Result<()> {
        unsafe {
            let ret = ffi::sqlite3_step(self.statement.statement);
            match ret {
                ffi::SQLITE_DONE => Ok(()),
                ffi::SQLITE_ROW => Err(Error::execute_returned_results()),
                _ => Err(Error::new(ret)),
            }
        }
    }

    pub fn fetch(&mut self) -> Rows {
        Rows {
            statement: Some(self.statement),
        }
    }
}

pub struct Statement<'a> {
    statement: *mut ffi::Statement,
    marker: PhantomData<&'a Connection>,
}

impl Statement<'_> {
    pub fn parameter_index(&mut self, parameter: &str) -> Option<NonZeroI32> {
        let name = CString::new(parameter).unwrap();
        let index = unsafe { ffi::sqlite3_bind_parameter_index(self.statement, name.as_ptr()) };
        NonZeroI32::new(index)
    }

    pub fn query(&mut self) -> Query {
        Query { statement: self }
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        let _ret = unsafe { ffi::sqlite3_finalize(self.statement) };
    }
}

pub struct Row<'stmt> {
    statement: &'stmt Statement<'stmt>,
}

impl Row<'_> {
    pub fn column_i32(&mut self, column: i32) -> i32 {
        unsafe { ffi::sqlite3_column_int(self.statement.statement, column) }
    }

    pub fn column_i64(&mut self, column: i32) -> i64 {
        unsafe { ffi::sqlite3_column_int64(self.statement.statement, column) }
    }

    pub fn column_f64(&mut self, column: i32) -> f64 {
        unsafe { ffi::sqlite3_column_double(self.statement.statement, column) }
    }

    pub fn column_str(&mut self, column: i32) -> &str {
        // Safety: Text is utf-8 and the pointer given by sqlite3_column_text is valid
        // until the statement is invalidated, or the column is cast to a different type
        // by another column accessor.
        //
        // Behave conservatively here and only allow exclusive access to column data.
        unsafe {
            let len = ffi::sqlite3_column_bytes(self.statement.statement, column)
                .try_into()
                .unwrap();
            let ptr = ffi::sqlite3_column_text(self.statement.statement, column);
            let slice = std::slice::from_raw_parts(ptr, len);
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn column_blob(&mut self, column: i32) -> &[u8] {
        // Safety: The pointer given by sqlite3_column_blob is valid until the statement
        // is invalidated, or the column is cast to a different type by another column
        // accessor.
        //
        // Behave conservatively here and only allow exclusive access to column data.
        unsafe {
            let len = ffi::sqlite3_column_bytes(self.statement.statement, column)
                .try_into()
                .unwrap();
            let ptr = ffi::sqlite3_column_blob(self.statement.statement, column) as *const u8;
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

pub struct Rows<'stmt> {
    statement: Option<&'stmt Statement<'stmt>>,
}

impl<'stmt> Iterator for Rows<'stmt> {
    type Item = Row<'stmt>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(statement) = self.statement {
            let ret = unsafe { ffi::sqlite3_step(statement.statement) };
            match ret {
                ffi::SQLITE_ROW => Some(Row { statement }),
                ffi::SQLITE_DONE => {
                    let _ret = unsafe { ffi::sqlite3_reset(statement.statement) };
                    self.statement = None;
                    None
                }
                _ => panic!("{:?}", Error::new(ret)),
            }
        } else {
            None
        }
    }
}

impl Drop for Rows<'_> {
    fn drop(&mut self) {
        if let Some(statement) = self.statement.take() {
            let _ = unsafe { ffi::sqlite3_reset(statement.statement) };
        }
    }
}

pub fn version() -> i32 {
    unsafe { ffi::sqlite3_libversion_number() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let connection = Connection::open_memory().unwrap();

        connection
            .execute("CREATE TABLE users (name TEXT, age INTEGER)")
            .unwrap();

        let mut statement = connection
            .prepare("INSERT INTO users VALUES ($name , $age)")
            .unwrap();

        let name_index = statement.parameter_index("$name").unwrap().get();
        let age_index = statement.parameter_index("$age").unwrap().get();

        let names_to_ages = [
            ("Grag", 11),
            ("Gerold", 23),
            ("Gerty", 20),
            ("Guillaume", 45),
            ("Geoff", 32),
        ];

        for (name, age) in names_to_ages {
            statement
                .query()
                .bind_text(name_index, name)
                .bind_i32(age_index, age)
                .execute()
                .unwrap();
        }

        let mut statement = connection
            .prepare("SELECT * from users ORDER BY rowid")
            .unwrap();

        for (&(name, age), mut row) in names_to_ages.iter().zip(statement.query().fetch()) {
            let db_age = row.column_i32(1);
            let db_name = row.column_str(0);
            assert_eq!(name, db_name);
            assert_eq!(age, db_age);
        }
    }
}
