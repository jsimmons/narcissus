use std::ffi::{c_char, c_int, c_uchar, c_void};

#[repr(C)]
pub struct Connection {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct Statement {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct Value {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Successful result
pub const SQLITE_OK: c_int = 0;
/// Generic error
pub const SQLITE_ERROR: c_int = 1;
/// Internal logic error in SQLite
pub const SQLITE_INTERNAL: c_int = 2;
/// Access permission denied
pub const SQLITE_PERM: c_int = 3;
/// Callback routine requested an abort
pub const SQLITE_ABORT: c_int = 4;
/// The database file is locked
pub const SQLITE_BUSY: c_int = 5;
/// A table in the database is locked
pub const SQLITE_LOCKED: c_int = 6;
/// A malloc() failed
pub const SQLITE_NOMEM: c_int = 7;
/// Attempt to write a readonly database
pub const SQLITE_READONLY: c_int = 8;
/// Operation terminated by sqlite3_interrupt()
pub const SQLITE_INTERRUPT: c_int = 9;
/// Some kind of disk I/O error occurred
pub const SQLITE_IOERR: c_int = 10;
/// The database disk image is malformed
pub const SQLITE_CORRUPT: c_int = 11;
/// Unknown opcode in sqlite3_file_control()
pub const SQLITE_NOTFOUND: c_int = 12;
/// Insertion failed because database is full
pub const SQLITE_FULL: c_int = 13;
/// Unable to open the database file
pub const SQLITE_CANTOPEN: c_int = 14;
/// Database lock protocol error
pub const SQLITE_PROTOCOL: c_int = 15;
/// Internal use only
pub const SQLITE_EMPTY: c_int = 16;
/// The database schema changed
pub const SQLITE_SCHEMA: c_int = 17;
/// String or BLOB exceeds size limit
pub const SQLITE_TOOBIG: c_int = 18;
/// Abort due to constraint violation
pub const SQLITE_CONSTRAINT: c_int = 19;
/// Data type mismatch
pub const SQLITE_MISMATCH: c_int = 20;
/// Library used incorrectly
pub const SQLITE_MISUSE: c_int = 21;
/// Uses OS features not supported on host
pub const SQLITE_NOLFS: c_int = 22;
/// Authorization denied
pub const SQLITE_AUTH: c_int = 23;
/// Not used
pub const SQLITE_FORMAT: c_int = 24;
/// 2nd parameter to sqlite3_bind out of range
pub const SQLITE_RANGE: c_int = 25;
/// File opened that is not a database file
pub const SQLITE_NOTADB: c_int = 26;
/// Notifications from sqlite3_log()
pub const SQLITE_NOTICE: c_int = 27;
/// Warnings from sqlite3_log()
pub const SQLITE_WARNING: c_int = 28;
/// sqlite3_step() has another row ready
pub const SQLITE_ROW: c_int = 100;
/// sqlite3_step() has finished executing
pub const SQLITE_DONE: c_int = 101;

pub const SQLITE_ERROR_MISSING_COLLSEQ: c_int = SQLITE_ERROR | (1 << 8);
pub const SQLITE_ERROR_RETRY: c_int = SQLITE_ERROR | (2 << 8);
pub const SQLITE_ERROR_SNAPSHOT: c_int = SQLITE_ERROR | (3 << 8);
pub const SQLITE_IOERR_READ: c_int = SQLITE_IOERR | (1 << 8);
pub const SQLITE_IOERR_SHORT_READ: c_int = SQLITE_IOERR | (2 << 8);
pub const SQLITE_IOERR_WRITE: c_int = SQLITE_IOERR | (3 << 8);
pub const SQLITE_IOERR_FSYNC: c_int = SQLITE_IOERR | (4 << 8);
pub const SQLITE_IOERR_DIR_FSYNC: c_int = SQLITE_IOERR | (5 << 8);
pub const SQLITE_IOERR_TRUNCATE: c_int = SQLITE_IOERR | (6 << 8);
pub const SQLITE_IOERR_FSTAT: c_int = SQLITE_IOERR | (7 << 8);
pub const SQLITE_IOERR_UNLOCK: c_int = SQLITE_IOERR | (8 << 8);
pub const SQLITE_IOERR_RDLOCK: c_int = SQLITE_IOERR | (9 << 8);
pub const SQLITE_IOERR_DELETE: c_int = SQLITE_IOERR | (10 << 8);
pub const SQLITE_IOERR_BLOCKED: c_int = SQLITE_IOERR | (11 << 8);
pub const SQLITE_IOERR_NOMEM: c_int = SQLITE_IOERR | (12 << 8);
pub const SQLITE_IOERR_ACCESS: c_int = SQLITE_IOERR | (13 << 8);
pub const SQLITE_IOERR_CHECKRESERVEDLOCK: c_int = SQLITE_IOERR | (14 << 8);
pub const SQLITE_IOERR_LOCK: c_int = SQLITE_IOERR | (15 << 8);
pub const SQLITE_IOERR_CLOSE: c_int = SQLITE_IOERR | (16 << 8);
pub const SQLITE_IOERR_DIR_CLOSE: c_int = SQLITE_IOERR | (17 << 8);
pub const SQLITE_IOERR_SHMOPEN: c_int = SQLITE_IOERR | (18 << 8);
pub const SQLITE_IOERR_SHMSIZE: c_int = SQLITE_IOERR | (19 << 8);
pub const SQLITE_IOERR_SHMLOCK: c_int = SQLITE_IOERR | (20 << 8);
pub const SQLITE_IOERR_SHMMAP: c_int = SQLITE_IOERR | (21 << 8);
pub const SQLITE_IOERR_SEEK: c_int = SQLITE_IOERR | (22 << 8);
pub const SQLITE_IOERR_DELETE_NOENT: c_int = SQLITE_IOERR | (23 << 8);
pub const SQLITE_IOERR_MMAP: c_int = SQLITE_IOERR | (24 << 8);
pub const SQLITE_IOERR_GETTEMPPATH: c_int = SQLITE_IOERR | (25 << 8);
pub const SQLITE_IOERR_CONVPATH: c_int = SQLITE_IOERR | (26 << 8);
pub const SQLITE_IOERR_VNODE: c_int = SQLITE_IOERR | (27 << 8);
pub const SQLITE_IOERR_AUTH: c_int = SQLITE_IOERR | (28 << 8);
pub const SQLITE_IOERR_BEGIN_ATOMIC: c_int = SQLITE_IOERR | (29 << 8);
pub const SQLITE_IOERR_COMMIT_ATOMIC: c_int = SQLITE_IOERR | (30 << 8);
pub const SQLITE_IOERR_ROLLBACK_ATOMIC: c_int = SQLITE_IOERR | (31 << 8);
pub const SQLITE_IOERR_DATA: c_int = SQLITE_IOERR | (32 << 8);
pub const SQLITE_IOERR_CORRUPTFS: c_int = SQLITE_IOERR | (33 << 8);
pub const SQLITE_LOCKED_SHAREDCACHE: c_int = SQLITE_LOCKED | (1 << 8);
pub const SQLITE_LOCKED_VTAB: c_int = SQLITE_LOCKED | (2 << 8);
pub const SQLITE_BUSY_RECOVERY: c_int = SQLITE_BUSY | (1 << 8);
pub const SQLITE_BUSY_SNAPSHOT: c_int = SQLITE_BUSY | (2 << 8);
pub const SQLITE_BUSY_TIMEOUT: c_int = SQLITE_BUSY | (3 << 8);
pub const SQLITE_CANTOPEN_NOTEMPDIR: c_int = SQLITE_CANTOPEN | (1 << 8);
pub const SQLITE_CANTOPEN_ISDIR: c_int = SQLITE_CANTOPEN | (2 << 8);
pub const SQLITE_CANTOPEN_FULLPATH: c_int = SQLITE_CANTOPEN | (3 << 8);
pub const SQLITE_CANTOPEN_CONVPATH: c_int = SQLITE_CANTOPEN | (4 << 8);
pub const SQLITE_CANTOPEN_DIRTYWAL: c_int = SQLITE_CANTOPEN | (5 << 8);
pub const SQLITE_CANTOPEN_SYMLINK: c_int = SQLITE_CANTOPEN | (6 << 8);
pub const SQLITE_CORRUPT_VTAB: c_int = SQLITE_CORRUPT | (1 << 8);
pub const SQLITE_CORRUPT_SEQUENCE: c_int = SQLITE_CORRUPT | (2 << 8);
pub const SQLITE_CORRUPT_INDEX: c_int = SQLITE_CORRUPT | (3 << 8);
pub const SQLITE_READONLY_RECOVERY: c_int = SQLITE_READONLY | (1 << 8);
pub const SQLITE_READONLY_CANTLOCK: c_int = SQLITE_READONLY | (2 << 8);
pub const SQLITE_READONLY_ROLLBACK: c_int = SQLITE_READONLY | (3 << 8);
pub const SQLITE_READONLY_DBMOVED: c_int = SQLITE_READONLY | (4 << 8);
pub const SQLITE_READONLY_CANTINIT: c_int = SQLITE_READONLY | (5 << 8);
pub const SQLITE_READONLY_DIRECTORY: c_int = SQLITE_READONLY | (6 << 8);
pub const SQLITE_ABORT_ROLLBACK: c_int = SQLITE_ABORT | (2 << 8);
pub const SQLITE_CONSTRAINT_CHECK: c_int = SQLITE_CONSTRAINT | (1 << 8);
pub const SQLITE_CONSTRAINT_COMMITHOOK: c_int = SQLITE_CONSTRAINT | (2 << 8);
pub const SQLITE_CONSTRAINT_FOREIGNKEY: c_int = SQLITE_CONSTRAINT | (3 << 8);
pub const SQLITE_CONSTRAINT_FUNCTION: c_int = SQLITE_CONSTRAINT | (4 << 8);
pub const SQLITE_CONSTRAINT_NOTNULL: c_int = SQLITE_CONSTRAINT | (5 << 8);
pub const SQLITE_CONSTRAINT_PRIMARYKEY: c_int = SQLITE_CONSTRAINT | (6 << 8);
pub const SQLITE_CONSTRAINT_TRIGGER: c_int = SQLITE_CONSTRAINT | (7 << 8);
pub const SQLITE_CONSTRAINT_UNIQUE: c_int = SQLITE_CONSTRAINT | (8 << 8);
pub const SQLITE_CONSTRAINT_VTAB: c_int = SQLITE_CONSTRAINT | (9 << 8);
pub const SQLITE_CONSTRAINT_ROWID: c_int = SQLITE_CONSTRAINT | (10 << 8);
pub const SQLITE_CONSTRAINT_PINNED: c_int = SQLITE_CONSTRAINT | (11 << 8);
pub const SQLITE_CONSTRAINT_DATATYPE: c_int = SQLITE_CONSTRAINT | (12 << 8);
pub const SQLITE_NOTICE_RECOVER_WAL: c_int = SQLITE_NOTICE | (1 << 8);
pub const SQLITE_NOTICE_RECOVER_ROLLBACK: c_int = SQLITE_NOTICE | (2 << 8);
pub const SQLITE_NOTICE_RBU: c_int = SQLITE_NOTICE | (3 << 8);
pub const SQLITE_WARNING_AUTOINDEX: c_int = SQLITE_WARNING | (1 << 8);
pub const SQLITE_AUTH_USER: c_int = SQLITE_AUTH | (1 << 8);
pub const SQLITE_OK_LOAD_PERMANENTLY: c_int = SQLITE_OK | (1 << 8);
pub const SQLITE_OK_SYMLINK: c_int = SQLITE_OK | (2 << 8);

/// Args: nil
pub const SQLITE_CONFIG_SINGLETHREAD: c_int = 1;
/// Args: nil
pub const SQLITE_CONFIG_MULTITHREAD: c_int = 2;
/// Args: nil
pub const SQLITE_CONFIG_SERIALIZED: c_int = 3;
/// Args: sqlite3_mem_methods*
pub const SQLITE_CONFIG_MALLOC: c_int = 4;
/// Args: sqlite3_mem_methods*
pub const SQLITE_CONFIG_GETMALLOC: c_int = 5;
/// Args: No longer used
pub const SQLITE_CONFIG_SCRATCH: c_int = 6;
/// Args: void*, int sz, int N
pub const SQLITE_CONFIG_PAGECACHE: c_int = 7;
/// Args: void*, int nByte, int min
pub const SQLITE_CONFIG_HEAP: c_int = 8;
/// Args: boolean
pub const SQLITE_CONFIG_MEMSTATUS: c_int = 9;
/// Args: sqlite3_mutex_methods*
pub const SQLITE_CONFIG_MUTEX: c_int = 10;
/// Args: sqlite3_mutex_methods*
pub const SQLITE_CONFIG_GETMUTEX: c_int = 11;
/// Args: int int
pub const SQLITE_CONFIG_LOOKASIDE: c_int = 13;
/// Args: no-op
pub const SQLITE_CONFIG_PCACHE: c_int = 14;
/// Args: no-op
pub const SQLITE_CONFIG_GETPCACHE: c_int = 15;
/// Args: xFunc, void*
pub const SQLITE_CONFIG_LOG: c_int = 16;
/// Args: int
pub const SQLITE_CONFIG_URI: c_int = 17;
/// Args: sqlite3_pcache_methods2*
pub const SQLITE_CONFIG_PCACHE2: c_int = 18;
/// Args: sqlite3_pcache_methods2*
pub const SQLITE_CONFIG_GETPCACHE2: c_int = 19;
/// Args: int
pub const SQLITE_CONFIG_COVERING_INDEX_SCAN: c_int = 20;
/// Args: xSqllog, void*
pub const SQLITE_CONFIG_SQLLOG: c_int = 21;
/// Args: sqlite3_int64, sqlite3_int64
pub const SQLITE_CONFIG_MMAP_SIZE: c_int = 22;
/// Args: int nByte
pub const SQLITE_CONFIG_WIN32_HEAPSIZE: c_int = 23;
/// Args: int *psz
pub const SQLITE_CONFIG_PCACHE_HDRSZ: c_int = 24;
/// Args: unsigned int szPma
pub const SQLITE_CONFIG_PMASZ: c_int = 25;
/// Args: int nByte
pub const SQLITE_CONFIG_STMTJRNL_SPILL: c_int = 26;
/// Args: boolean
pub const SQLITE_CONFIG_SMALL_MALLOC: c_int = 27;
/// Args: int nByte
pub const SQLITE_CONFIG_SORTERREF_SIZE: c_int = 28;
/// Args: sqlite3_int64
pub const SQLITE_CONFIG_MEMDB_MAXSIZE: c_int = 29;

/// Args: const char*
pub const SQLITE_DBCONFIG_MAINDBNAME: c_int = 1000;
/// Args: void* int int
pub const SQLITE_DBCONFIG_LOOKASIDE: c_int = 1001;
/// Args: int int*
pub const SQLITE_DBCONFIG_ENABLE_FKEY: c_int = 1002;
/// Args: int int*
pub const SQLITE_DBCONFIG_ENABLE_TRIGGER: c_int = 1003;
/// Args: int int*
pub const SQLITE_DBCONFIG_ENABLE_FTS3_TOKENIZER: c_int = 1004;
/// Args: int int*
pub const SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION: c_int = 1005;
/// Args: int int*
pub const SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE: c_int = 1006;
/// Args: int int*
pub const SQLITE_DBCONFIG_ENABLE_QPSG: c_int = 1007;
/// Args: int int*
pub const SQLITE_DBCONFIG_TRIGGER_EQP: c_int = 1008;
/// Args: int int*
pub const SQLITE_DBCONFIG_RESET_DATABASE: c_int = 1009;
/// Args: int int*
pub const SQLITE_DBCONFIG_DEFENSIVE: c_int = 1010;
/// Args: int int*
pub const SQLITE_DBCONFIG_WRITABLE_SCHEMA: c_int = 1011;
/// Args: int int*
pub const SQLITE_DBCONFIG_LEGACY_ALTER_TABLE: c_int = 1012;
/// Args: int int*
pub const SQLITE_DBCONFIG_DQS_DML: c_int = 1013;
/// Args: int int*
pub const SQLITE_DBCONFIG_DQS_DDL: c_int = 1014;
/// Args: int int*
pub const SQLITE_DBCONFIG_ENABLE_VIEW: c_int = 1015;
/// Args: int int*
pub const SQLITE_DBCONFIG_LEGACY_FILE_FORMAT: c_int = 1016;
/// Args: int int*
pub const SQLITE_DBCONFIG_TRUSTED_SCHEMA: c_int = 1017;
/// Args: int int*
pub const SQLITE_DBCONFIG_STMT_SCANSTATUS: c_int = 1018;
/// Args: int int*
pub const SQLITE_DBCONFIG_REVERSE_SCANORDER: c_int = 1019;
/// Args: Largest DBCONFIG
pub const SQLITE_DBCONFIG_MAX: c_int = 1019;

/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_READONLY: c_int = 0x00000001;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_READWRITE: c_int = 0x00000002;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_CREATE: c_int = 0x00000004;
/// VFS only
pub const SQLITE_OPEN_DELETEONCLOSE: c_int = 0x00000008;
/// VFS only
pub const SQLITE_OPEN_EXCLUSIVE: c_int = 0x00000010;
/// VFS only
pub const SQLITE_OPEN_AUTOPROXY: c_int = 0x00000020;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_URI: c_int = 0x00000040;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_MEMORY: c_int = 0x00000080;
/// VFS only
pub const SQLITE_OPEN_MAIN_DB: c_int = 0x00000100;
/// VFS only
pub const SQLITE_OPEN_TEMP_DB: c_int = 0x00000200;
/// VFS only
pub const SQLITE_OPEN_TRANSIENT_DB: c_int = 0x00000400;
/// VFS only
pub const SQLITE_OPEN_MAIN_JOURNAL: c_int = 0x00000800;
/// VFS only
pub const SQLITE_OPEN_TEMP_JOURNAL: c_int = 0x00001000;
/// VFS only
pub const SQLITE_OPEN_SUBJOURNAL: c_int = 0x00002000;
/// VFS only
pub const SQLITE_OPEN_SUPER_JOURNAL: c_int = 0x00004000;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_NOMUTEX: c_int = 0x00008000;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_FULLMUTEX: c_int = 0x00010000;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_SHAREDCACHE: c_int = 0x00020000;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_PRIVATECACHE: c_int = 0x00040000;
/// VFS only
pub const SQLITE_OPEN_WAL: c_int = 0x00080000;
/// Ok for sqlite3_open_v2()
pub const SQLITE_OPEN_NOFOLLOW: c_int = 0x01000000;
/// Extended result codes
pub const SQLITE_OPEN_EXRESCODE: c_int = 0x02000000;

pub const SQLITE_INTEGER: c_int = 1;
pub const SQLITE_FLOAT: c_int = 2;
pub const SQLITE_TEXT: c_int = 3;
pub const SQLITE_BLOB: c_int = 4;
pub const SQLITE_NULL: c_int = 5;

pub const SQLITE_UTF8: c_uchar = 1;

#[repr(C)]
pub struct Destructor(*mut c_void);

impl Destructor {
    pub fn new(fun: unsafe extern "C" fn(*mut c_void)) -> Self {
        unsafe { std::mem::transmute(fun) }
    }
}

pub const SQLITE_STATIC: Destructor = unsafe { std::mem::transmute(0usize) };
pub const SQLITE_TRANSIENT: Destructor = unsafe { std::mem::transmute(!0usize) };

extern "C" {
    pub fn sqlite3_libversion_number() -> c_int;

    pub fn sqlite3_initialize() -> c_int;

    pub fn sqlite3_config(op: c_int, ...) -> c_int;
    pub fn sqlite3_db_config(connection: *mut Connection, op: c_int, ...) -> c_int;

    pub fn sqlite3_open_v2(
        filename: *const c_char,
        connection: *mut *mut Connection,
        flags: c_int,
        vfs: *const c_char,
    ) -> c_int;

    pub fn sqlite3_close(connection: *mut Connection) -> c_int;

    pub fn sqlite3_prepare_v2(
        connection: *mut Connection,
        sql: *const c_char,
        sql_len: c_int,
        statement: *mut *mut Statement,
        tail: *mut *const c_char,
    ) -> c_int;

    pub fn sqlite3_step(statement: *mut Statement) -> c_int;
    pub fn sqlite3_reset(statement: *mut Statement) -> c_int;
    pub fn sqlite3_clear_bindings(statement: *mut Statement) -> c_int;
    pub fn sqlite3_finalize(statement: *mut Statement) -> c_int;

    pub fn sqlite3_bind_parameter_index(statement: *mut Statement, name: *const c_char) -> c_int;
    pub fn sqlite3_bind_blob(
        statement: *mut Statement,
        index: c_int,
        blob: *const c_void,
        len: c_int,
        destructor: Destructor,
    );
    pub fn sqlite3_bind_blob64(
        statement: *mut Statement,
        index: c_int,
        blob: *const c_void,
        len: u64,
        destructor: Destructor,
    );
    pub fn sqlite3_bind_double(statement: *mut Statement, index: c_int, value: f64);
    pub fn sqlite3_bind_int(statement: *mut Statement, index: c_int, value: c_int);
    pub fn sqlite3_bind_int64(statement: *mut Statement, index: c_int, value: i64);
    pub fn sqlite3_bind_null(statement: *mut Statement, index: c_int);
    pub fn sqlite3_bind_text(
        statement: *mut Statement,
        index: c_int,
        data: *const c_char,
        len: c_int,
        destructor: Destructor,
    );
    pub fn sqlite3_bind_text64(
        statement: *mut Statement,
        index: c_int,
        data: *const c_char,
        len: u64,
        destructor: Destructor,
        encoding: c_uchar,
    );
    pub fn sqlite3_bind_value(statement: *mut Statement, index: c_int, value: *const Value);
    pub fn sqlite3_bind_pointer(
        statement: *mut Statement,
        index: c_int,
        ptr: *mut c_void,
        ty: *const c_char,
        destructor: Destructor,
    );
    pub fn sqlite3_bind_zeroblob(statement: *mut Statement, index: c_int, len: c_int);
    pub fn sqlite3_bind_zeroblob64(statement: *mut Statement, index: c_int, len: u64);

    pub fn sqlite3_column_count(statement: *mut Statement) -> i32;
    pub fn sqlite3_column_blob(statement: *mut Statement, column: c_int) -> *const c_void;
    pub fn sqlite3_column_double(statement: *mut Statement, column: c_int) -> f64;
    pub fn sqlite3_column_int(statement: *mut Statement, column: c_int) -> c_int;
    pub fn sqlite3_column_int64(statement: *mut Statement, column: c_int) -> i64;
    pub fn sqlite3_column_text(statement: *mut Statement, column: c_int) -> *const c_uchar;
    pub fn sqlite3_column_bytes(statement: *mut Statement, column: c_int) -> c_int;
    pub fn sqlite3_column_type(statement: *mut Statement, column: c_int) -> c_int;
}
