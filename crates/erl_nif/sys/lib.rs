#![allow(clippy::all, non_camel_case_types, non_snake_case)]

use erl_nif_macro::api;

use std::os::raw::{c_char, c_double, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};
type size_t = usize;

pub const ERL_NIF_MAJOR_VERSION: c_int = 2;
pub const ERL_NIF_MINOR_VERSION: c_int = 15;

pub const ERL_NAPI_SINT64_MAX__: u64 = 9223372036854775807;
pub const ERL_NAPI_SINT64_MIN__: i64 = -9223372036854775808;
pub const ERL_NIF_IOVEC_SIZE: u32 = 16;
pub const ERL_NIF_MIN_ERTS_VERSION: &'static [u8; 10usize] = b"erts-10.4\0";
pub const ERL_NIF_MIN_REQUIRED_MAJOR_VERSION_ON_LOAD: c_int = 2;
pub const ERL_NIF_SELECT_FAILED: u32 = 8;
pub const ERL_NIF_SELECT_INVALID_EVENT: u32 = 4;
pub const ERL_NIF_SELECT_READ_CANCELLED: u32 = 16;
pub const ERL_NIF_SELECT_STOP_CALLED: u32 = 1;
pub const ERL_NIF_SELECT_STOP_SCHEDULED: u32 = 2;
pub const ERL_NIF_SELECT_WRITE_CANCELLED: u32 = 32;
pub const ERL_NIF_THR_DIRTY_CPU_SCHEDULER: u32 = 2;
pub const ERL_NIF_THR_DIRTY_IO_SCHEDULER: u32 = 3;
pub const ERL_NIF_THR_NORMAL_SCHEDULER: u32 = 1;
pub const ERL_NIF_THR_UNDEFINED: u32 = 0;
pub const ERL_NIF_VM_VARIANT: &'static [u8; 13usize] = b"beam.vanilla\0";
pub const ERTS_NAPI_MSEC__: u32 = 1;
pub const ERTS_NAPI_NSEC__: u32 = 3;
pub const ERTS_NAPI_SEC__: u32 = 0;
pub const ERTS_NAPI_TIME_ERROR__: i64 = -9223372036854775808;
pub const ERTS_NAPI_USEC__: u32 = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvSysInfo {
	pub driver_major_version: c_int,
	pub driver_minor_version: c_int,
	pub erts_version: *mut c_char,
	pub otp_release: *mut c_char,
	pub thread_support: c_int,
	pub smp_support: c_int,
	pub async_threads: c_int,
	pub scheduler_threads: c_int,
	pub nif_major_version: c_int,
	pub nif_minor_version: c_int,
	pub dirty_scheduler_support: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvThreadOpts {
	pub suggested_stack_size: c_int,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlDirtyJobFlags {
	ERL_DIRTY_JOB_CPU_BOUND = 1,
	ERL_DIRTY_JOB_IO_BOUND = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifSelectFlags {
	ERL_NIF_SELECT_READ = 1,
	ERL_NIF_SELECT_WRITE = 2,
	ERL_NIF_SELECT_STOP = 4,
	ERL_NIF_SELECT_CANCEL = 8,
	ERL_NIF_SELECT_CUSTOM_MSG = 16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvMonitor {
	pub data: [c_uchar; 32usize],
}

pub type ERL_NIF_TERM = u64;
pub type ERL_NIF_UINT = ERL_NIF_TERM;
pub type ErlNifTime = i64;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifTimeUnit {
	ERL_NIF_SEC = 0,
	ERL_NIF_MSEC = 1,
	ERL_NIF_USEC = 2,
	ERL_NIF_NSEC = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct enif_environment_t {
	_unused: [u8; 0],
}

pub type ErlNifEnv = enif_environment_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct enif_func_t {
	pub name: *const c_char,
	pub arity: c_uint,
	pub fptr: Option<
		unsafe extern "C" fn(
			env: *mut ErlNifEnv,
			argc: c_int,
			argv: *const ERL_NIF_TERM,
		) -> ERL_NIF_TERM,
	>,
	pub flags: c_uint,
}

pub type ErlNifFunc = enif_func_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct enif_entry_t {
	pub major: c_int,
	pub minor: c_int,
	pub name: *const c_char,
	pub num_of_funcs: c_int,
	pub funcs: *const ErlNifFunc,
	pub load: Option<
		unsafe extern "C" fn(
			arg1: *mut ErlNifEnv,
			priv_data: *mut *mut c_void,
			load_info: ERL_NIF_TERM,
		) -> c_int,
	>,
	pub reload: Option<
		unsafe extern "C" fn(
			arg1: *mut ErlNifEnv,
			priv_data: *mut *mut c_void,
			load_info: ERL_NIF_TERM,
		) -> c_int,
	>,
	pub upgrade: Option<
		unsafe extern "C" fn(
			arg1: *mut ErlNifEnv,
			priv_data: *mut *mut c_void,
			old_priv_data: *mut *mut c_void,
			load_info: ERL_NIF_TERM,
		) -> c_int,
	>,
	pub unload: Option<unsafe extern "C" fn(arg1: *mut ErlNifEnv, priv_data: *mut c_void)>,
	pub vm_variant: *const c_char,
	pub options: c_uint,
	pub sizeof_ErlNifResourceTypeInit: size_t,
	pub min_erts: *const c_char,
}

pub type ErlNifEntry = enif_entry_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlNifBinary {
	pub size: size_t,
	pub data: *mut c_uchar,
	pub ref_bin: *mut c_void,
	pub __spare__: [*mut c_void; 2usize],
}

pub type ErlNifEvent = c_int;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifResourceFlags {
	ERL_NIF_RT_CREATE = 1,
	ERL_NIF_RT_TAKEOVER = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifCharEncoding {
	ERL_NIF_LATIN1 = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlNifPid {
	pub pid: ERL_NIF_TERM,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlNifPort {
	pub port_id: ERL_NIF_TERM,
}

pub type ErlNifMonitor = ErlDrvMonitor;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct enif_resource_type_t {
	_unused: [u8; 0],
}

pub type ErlNifResourceType = enif_resource_type_t;
pub type ErlNifResourceDtor = Option<unsafe extern "C" fn(arg1: *mut ErlNifEnv, arg2: *mut c_void)>;
pub type ErlNifResourceStop = Option<
	unsafe extern "C" fn(
		arg1: *mut ErlNifEnv,
		arg2: *mut c_void,
		arg3: ErlNifEvent,
		is_direct_call: c_int,
	),
>;
pub type ErlNifResourceDown = Option<
	unsafe extern "C" fn(
		arg1: *mut ErlNifEnv,
		arg2: *mut c_void,
		arg3: *mut ErlNifPid,
		arg4: *mut ErlNifMonitor,
	),
>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlNifResourceTypeInit {
	pub dtor: ErlNifResourceDtor,
	pub stop: ErlNifResourceStop,
	pub down: ErlNifResourceDown,
}

pub type ErlNifSysInfo = ErlDrvSysInfo;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvTid_ {
	_unused: [u8; 0],
}

pub type ErlNifTid = *mut ErlDrvTid_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvMutex_ {
	_unused: [u8; 0],
}

pub type ErlNifMutex = ErlDrvMutex_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvCond_ {
	_unused: [u8; 0],
}

pub type ErlNifCond = ErlDrvCond_;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlDrvRWLock_ {
	_unused: [u8; 0],
}

pub type ErlNifRWLock = ErlDrvRWLock_;
pub type ErlNifTSDKey = c_int;
pub type ErlNifThreadOpts = ErlDrvThreadOpts;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifDirtyTaskFlags {
	ERL_NIF_DIRTY_JOB_CPU_BOUND = 1,
	ERL_NIF_DIRTY_JOB_IO_BOUND = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ErlNifMapIterator {
	pub map: ERL_NIF_TERM,
	pub size: ERL_NIF_UINT,
	pub idx: ERL_NIF_UINT,
	pub u: ErlNifMapIterator__bindgen_ty_1,
	pub __spare__: [*mut c_void; 2usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ErlNifMapIterator__bindgen_ty_1 {
	pub flat: ErlNifMapIterator__bindgen_ty_1__bindgen_ty_1,
	pub hash: ErlNifMapIterator__bindgen_ty_1__bindgen_ty_2,
	_bindgen_union_align: [u64; 2usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlNifMapIterator__bindgen_ty_1__bindgen_ty_1 {
	pub ks: *mut ERL_NIF_TERM,
	pub vs: *mut ERL_NIF_TERM,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErlNifMapIterator__bindgen_ty_1__bindgen_ty_2 {
	pub wstack: *mut ErtsDynamicWStack_,
	pub kv: *mut ERL_NIF_TERM,
}

impl ErlNifMapIteratorEntry {
	pub const ERL_NIF_MAP_ITERATOR_HEAD: ErlNifMapIteratorEntry =
		ErlNifMapIteratorEntry::ERL_NIF_MAP_ITERATOR_FIRST;
}

impl ErlNifMapIteratorEntry {
	pub const ERL_NIF_MAP_ITERATOR_TAIL: ErlNifMapIteratorEntry =
		ErlNifMapIteratorEntry::ERL_NIF_MAP_ITERATOR_LAST;
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifMapIteratorEntry {
	ERL_NIF_MAP_ITERATOR_FIRST = 1,
	ERL_NIF_MAP_ITERATOR_LAST = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifUniqueInteger {
	ERL_NIF_UNIQUE_POSITIVE = 1,
	ERL_NIF_UNIQUE_MONOTONIC = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifBinaryToTerm {
	ERL_NIF_BIN2TERM_SAFE = 536870912,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifHash {
	ERL_NIF_INTERNAL_HASH = 1,
	ERL_NIF_PHASH2 = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct erl_nif_io_vec {
	pub iovcnt: c_int,
	pub size: size_t,
	pub iov: *mut SysIOVec,
	pub ref_bins: *mut *mut c_void,
	pub flags: c_int,
	pub small_iov: [SysIOVec; 16usize],
	pub small_ref_bin: [*mut c_void; 16usize],
}

pub type ErlNifIOVec = erl_nif_io_vec;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct erts_io_queue {
	_unused: [u8; 0],
}

pub type ErlNifIOQueue = erts_io_queue;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifIOQueueOpts {
	ERL_NIF_IOQ_NORMAL = 1,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ErlNifTermType {
	ERL_NIF_TERM_TYPE_ATOM = 1,
	ERL_NIF_TERM_TYPE_BITSTRING = 2,
	ERL_NIF_TERM_TYPE_FLOAT = 3,
	ERL_NIF_TERM_TYPE_FUN = 4,
	ERL_NIF_TERM_TYPE_INTEGER = 5,
	ERL_NIF_TERM_TYPE_LIST = 6,
	ERL_NIF_TERM_TYPE_MAP = 7,
	ERL_NIF_TERM_TYPE_PID = 8,
	ERL_NIF_TERM_TYPE_PORT = 9,
	ERL_NIF_TERM_TYPE_REFERENCE = 10,
	ERL_NIF_TERM_TYPE_TUPLE = 11,
	ERL_NIF_TERM_TYPE__MISSING_DEFAULT_CASE__READ_THE_MANUAL = -1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErtsDynamicWStack_ {
	pub _address: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SysIOVec {
	pub iov_base: *mut c_char,
	pub iov_len: size_t,
}

#[rustfmt::skip]
api! {
	fn enif_priv_data(env: *mut ErlNifEnv) -> *mut c_void;
	fn enif_alloc(size: size_t) -> *mut c_void;
	fn enif_free(ptr: *mut c_void);
	fn enif_is_atom(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_is_binary(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_is_ref(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_inspect_binary(env: *mut ErlNifEnv, bin_term: ERL_NIF_TERM, bin: *mut ErlNifBinary) -> c_int;
	fn enif_alloc_binary(size: size_t, bin: *mut ErlNifBinary) -> c_int;
	fn enif_realloc_binary(bin: *mut ErlNifBinary, size: size_t) -> c_int;
	fn enif_release_binary(bin: *mut ErlNifBinary);
	fn enif_get_int(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut c_int) -> c_int;
	fn enif_get_ulong(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut c_ulong) -> c_int;
	fn enif_get_double(env: *mut ErlNifEnv, term: ERL_NIF_TERM, dp: *mut c_double) -> c_int;
	fn enif_get_list_cell(env: *mut ErlNifEnv, term: ERL_NIF_TERM, head: *mut ERL_NIF_TERM, tail: *mut ERL_NIF_TERM) -> c_int;
	fn enif_get_tuple(env: *mut ErlNifEnv, tpl: ERL_NIF_TERM, arity: *mut c_int, array: *mut *const ERL_NIF_TERM) -> c_int;
	fn enif_is_identical(lhs: ERL_NIF_TERM, rhs: ERL_NIF_TERM) -> c_int;
	fn enif_compare(lhs: ERL_NIF_TERM, rhs: ERL_NIF_TERM) -> c_int;
	fn enif_make_binary(env: *mut ErlNifEnv, bin: *mut ErlNifBinary) -> ERL_NIF_TERM;
	fn enif_make_badarg(env: *mut ErlNifEnv) -> ERL_NIF_TERM;
	fn enif_make_int(env: *mut ErlNifEnv, i: c_int) -> ERL_NIF_TERM;
	fn enif_make_ulong(env: *mut ErlNifEnv, i: c_ulong) -> ERL_NIF_TERM;
	fn enif_make_double(env: *mut ErlNifEnv, d: c_double) -> ERL_NIF_TERM;
	fn enif_make_atom(env: *mut ErlNifEnv, name: *const c_char) -> ERL_NIF_TERM;
	fn enif_make_existing_atom(env: *mut ErlNifEnv, name: *const c_char, atom: *mut ERL_NIF_TERM, encoding: ErlNifCharEncoding) -> c_int;
	fn enif_make_tuple(env: *mut ErlNifEnv, cnt: c_uint, ...) -> ERL_NIF_TERM;
	fn enif_make_list(env: *mut ErlNifEnv, cnt: c_uint, ...) -> ERL_NIF_TERM;
	fn enif_make_list_cell(env: *mut ErlNifEnv, car: ERL_NIF_TERM, cdr: ERL_NIF_TERM) -> ERL_NIF_TERM;
	fn enif_make_string(env: *mut ErlNifEnv, string: *const c_char, encoding: ErlNifCharEncoding) -> ERL_NIF_TERM;
	fn enif_make_ref(env: *mut ErlNifEnv) -> ERL_NIF_TERM;
	fn enif_mutex_create(name: *mut c_char) -> *mut ErlNifMutex;
	fn enif_mutex_destroy(mtx: *mut ErlNifMutex);
	fn enif_mutex_trylock(mtx: *mut ErlNifMutex) -> c_int;
	fn enif_mutex_lock(mtx: *mut ErlNifMutex);
	fn enif_mutex_unlock(mtx: *mut ErlNifMutex);
	fn enif_cond_create(name: *mut c_char) -> *mut ErlNifCond;
	fn enif_cond_destroy(cnd: *mut ErlNifCond);
	fn enif_cond_signal(cnd: *mut ErlNifCond);
	fn enif_cond_broadcast(cnd: *mut ErlNifCond);
	fn enif_cond_wait(cnd: *mut ErlNifCond, mtx: *mut ErlNifMutex);
	fn enif_rwlock_create(name: *mut c_char) -> *mut ErlNifRWLock;
	fn enif_rwlock_destroy(rwlck: *mut ErlNifRWLock);
	fn enif_rwlock_tryrlock(rwlck: *mut ErlNifRWLock) -> c_int;
	fn enif_rwlock_rlock(rwlck: *mut ErlNifRWLock);
	fn enif_rwlock_runlock(rwlck: *mut ErlNifRWLock);
	fn enif_rwlock_tryrwlock(rwlck: *mut ErlNifRWLock) -> c_int;
	fn enif_rwlock_rwlock(rwlck: *mut ErlNifRWLock);
	fn enif_rwlock_rwunlock(rwlck: *mut ErlNifRWLock);
	fn enif_tsd_key_create(name: *mut c_char, key: *mut ErlNifTSDKey) -> c_int;
	fn enif_tsd_key_destroy(key: ErlNifTSDKey);
	fn enif_tsd_set(key: ErlNifTSDKey, data: *mut c_void);
	fn enif_tsd_get(key: ErlNifTSDKey) -> *mut c_void;
	fn enif_thread_opts_create(name: *mut c_char) -> *mut ErlNifThreadOpts;
	fn enif_thread_opts_destroy(opts: *mut ErlNifThreadOpts);
	fn enif_thread_create(name: *mut c_char, tid: *mut ErlNifTid, func: Option<unsafe extern "C" fn(arg0: *mut c_void) -> *mut c_void>, args: *mut c_void, opts: *mut ErlNifThreadOpts) -> c_int;
	fn enif_thread_self() -> ErlNifTid;
	fn enif_equal_tids(tid1: ErlNifTid, tid2: ErlNifTid) -> c_int;
	fn enif_thread_exit(resp: *mut c_void);
	fn enif_thread_join(tid: ErlNifTid, respp: *mut *mut c_void) -> c_int;
	fn enif_realloc(ptr: *mut c_void, size: size_t) -> *mut c_void;
	fn enif_system_info(sip: *mut ErlNifSysInfo, si_size: size_t);
	fn enif_fprintf(); // fn enif_fprintf(FILE* filep, const format: *mut c_char, ...) -> c_int;
	fn enif_inspect_iolist_as_binary(env: *mut ErlNifEnv, term: ERL_NIF_TERM, bin: *mut ErlNifBinary) -> c_int;
	fn enif_make_sub_binary(env: *mut ErlNifEnv, bin_term: ERL_NIF_TERM, pos: size_t, size: size_t) -> ERL_NIF_TERM;
	fn enif_get_string(env: *mut ErlNifEnv, list: ERL_NIF_TERM, buf: *mut c_char, len: c_uint, encoding: ErlNifCharEncoding) -> c_int;
	fn enif_get_atom(env: *mut ErlNifEnv, atom: ERL_NIF_TERM, buf: *mut c_char, len: c_uint, encoding: ErlNifCharEncoding) -> c_int;
	fn enif_is_fun(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_is_pid(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_is_port(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_get_uint(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut c_uint) -> c_int;
	fn enif_get_long(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut c_long) -> c_int;
	fn enif_make_uint(env: *mut ErlNifEnv, i: c_uint) -> ERL_NIF_TERM;
	fn enif_make_long(env: *mut ErlNifEnv, i: c_long) -> ERL_NIF_TERM;
	fn enif_make_tuple_from_array(env: *mut ErlNifEnv, arr: *const ERL_NIF_TERM, cnt: c_uint) -> ERL_NIF_TERM;
	fn enif_make_list_from_array(env: *mut ErlNifEnv, arr: *const ERL_NIF_TERM, cnt: c_uint) -> ERL_NIF_TERM;
	fn enif_is_empty_list(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_open_resource_type(env: *mut ErlNifEnv, module_str: *const c_char, name_str: *const c_char, dtor: Option<unsafe extern "C" fn(arg1: *mut ErlNifEnv, arg2: *mut c_void)>, flags: ErlNifResourceFlags, tried: *mut ErlNifResourceFlags) -> *mut ErlNifResourceType;
	fn enif_alloc_resource(type_: *mut ErlNifResourceType, size: size_t) -> *mut c_void;
	fn enif_release_resource(obj: *mut c_void);
	fn enif_make_resource(env: *mut ErlNifEnv, obj: *mut c_void) -> ERL_NIF_TERM;
	fn enif_get_resource(env: *mut ErlNifEnv, term: ERL_NIF_TERM, type_: *mut ErlNifResourceType, objp: *mut *mut c_void) -> c_int;
	fn enif_sizeof_resource(obj: *mut c_void) -> size_t;
	fn enif_make_new_binary(env: *mut ErlNifEnv, size: size_t, termp: *mut ERL_NIF_TERM) -> *mut c_uchar;
	fn enif_is_list(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_is_tuple(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_get_atom_length(env: *mut ErlNifEnv, atom: ERL_NIF_TERM, len: *mut c_uint, encoding: ErlNifCharEncoding) -> c_int;
	fn enif_get_list_length(env: *mut ErlNifEnv, term: ERL_NIF_TERM, len: *mut c_uint) -> c_int;
	fn enif_make_atom_len(env: *mut ErlNifEnv, name: *const c_char, len: size_t) -> ERL_NIF_TERM;
	fn enif_make_existing_atom_len(env: *mut ErlNifEnv, name: *const c_char, len: size_t, atom: *mut ERL_NIF_TERM, encoding: ErlNifCharEncoding) -> c_int;
	fn enif_make_string_len(env: *mut ErlNifEnv, string: *const c_char, len: size_t, encoding: ErlNifCharEncoding) -> ERL_NIF_TERM;
	fn enif_alloc_env() -> *mut ErlNifEnv;
	fn enif_free_env(env: *mut ErlNifEnv);
	fn enif_clear_env(env: *mut ErlNifEnv);
	fn enif_send(env: *mut ErlNifEnv, to_pid: *const ErlNifPid, msg_env: *mut ErlNifEnv, msg: ERL_NIF_TERM) -> c_int;
	fn enif_make_copy(dst_env: *mut ErlNifEnv, src_term: ERL_NIF_TERM) -> ERL_NIF_TERM;
	fn enif_self(caller_env: *mut ErlNifEnv, pid: *mut ErlNifPid) -> *mut ErlNifPid;
	fn enif_get_local_pid(env: *mut ErlNifEnv, term: ERL_NIF_TERM, pid: *mut ErlNifPid) -> c_int;
	fn enif_keep_resource(obj: *mut c_void);
	fn enif_make_resource_binary(env: *mut ErlNifEnv, obj: *mut c_void, data: *const c_void, size: size_t) -> ERL_NIF_TERM;
	#[cfg(windows)]
	fn enif_get_int64(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut i64) -> c_int;
	#[cfg(windows)]
	fn enif_get_uint64(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut u64) -> c_int;
	#[cfg(windows)]
	fn enif_make_int64(env: *mut ErlNifEnv, i: i64) -> ERL_NIF_TERM;
	#[cfg(windows)]
	fn enif_make_uint64(env: *mut ErlNifEnv, i: u64) -> ERL_NIF_TERM;
	fn enif_is_exception(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_make_reverse_list(env: *mut ErlNifEnv, term: ERL_NIF_TERM, list: *mut ERL_NIF_TERM) -> c_int;
	fn enif_is_number(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_dlopen(lib: *const c_char, err_handler: Option<unsafe extern "C" fn(arg1: *mut c_void, arg2: *const c_char)>, err_arg: *mut c_void) -> *mut c_void;
	fn enif_dlsym(handle: *mut c_void, symbol: *const c_char, err_handler: Option<unsafe extern "C" fn(arg1: *mut c_void, arg2: *const c_char)>, err_arg: *mut c_void) -> *mut c_void;
	fn enif_consume_timeslice(env: *mut ErlNifEnv, percent: c_int) -> c_int;
	fn enif_is_map(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> c_int;
	fn enif_get_map_size(env: *mut ErlNifEnv, term: ERL_NIF_TERM, size: *mut size_t) -> c_int;
	fn enif_make_new_map(env: *mut ErlNifEnv) -> ERL_NIF_TERM;
	fn enif_make_map_put(env: *mut ErlNifEnv, map_in: ERL_NIF_TERM, key: ERL_NIF_TERM, value: ERL_NIF_TERM, map_out: *mut ERL_NIF_TERM) -> c_int;
	fn enif_get_map_value(env: *mut ErlNifEnv, map: ERL_NIF_TERM, key: ERL_NIF_TERM, value: *mut ERL_NIF_TERM) -> c_int;
	fn enif_make_map_update(env: *mut ErlNifEnv, map_in: ERL_NIF_TERM, key: ERL_NIF_TERM, value: ERL_NIF_TERM, map_out: *mut ERL_NIF_TERM) -> c_int;
	fn enif_make_map_remove(env: *mut ErlNifEnv, map_in: ERL_NIF_TERM, key: ERL_NIF_TERM, map_out: *mut ERL_NIF_TERM) -> c_int;
	fn enif_map_iterator_create(env: *mut ErlNifEnv, map: ERL_NIF_TERM, iter: *mut ErlNifMapIterator, entry: ErlNifMapIteratorEntry) -> c_int;
	fn enif_map_iterator_destroy(env: *mut ErlNifEnv, iter: *mut ErlNifMapIterator);
	fn enif_map_iterator_is_head(env: *mut ErlNifEnv, iter: *mut ErlNifMapIterator) -> c_int;
	fn enif_map_iterator_is_tail(env: *mut ErlNifEnv, iter: *mut ErlNifMapIterator) -> c_int;
	fn enif_map_iterator_next(env: *mut ErlNifEnv, iter: *mut ErlNifMapIterator) -> c_int;
	fn enif_map_iterator_prev(env: *mut ErlNifEnv, iter: *mut ErlNifMapIterator) -> c_int;
	fn enif_map_iterator_get_pair(env: *mut ErlNifEnv, iter: *mut ErlNifMapIterator, key: *mut ERL_NIF_TERM, value: *mut ERL_NIF_TERM) -> c_int;
	fn enif_schedule_nif(arg1: *mut ErlNifEnv, arg2: *const c_char, arg3: c_int, arg4: Option<unsafe extern "C" fn(arg1: *mut ErlNifEnv, arg2: c_int, arg3: *const ERL_NIF_TERM) -> ERL_NIF_TERM>, arg5: c_int, arg6: *const ERL_NIF_TERM) -> ERL_NIF_TERM;
	fn enif_has_pending_exception(env: *mut ErlNifEnv, reason: *mut ERL_NIF_TERM) -> c_int;
	fn enif_raise_exception(env: *mut ErlNifEnv, reason: ERL_NIF_TERM) -> ERL_NIF_TERM;
	fn enif_getenv(key: *const c_char, value: *mut c_char, value_size: *mut size_t) -> c_int;
	fn enif_monotonic_time(arg1: ErlNifTimeUnit) -> ErlNifTime;
	fn enif_time_offset(arg1: ErlNifTimeUnit) -> ErlNifTime;
	fn enif_convert_time_unit(arg1: ErlNifTime, arg2: ErlNifTimeUnit, arg3: ErlNifTimeUnit) -> ErlNifTime;
	fn enif_now_time(env: *mut ErlNifEnv) -> ERL_NIF_TERM;
	fn enif_cpu_time(env: *mut ErlNifEnv) -> ERL_NIF_TERM;
	fn enif_make_unique_integer(env: *mut ErlNifEnv, properties: ErlNifUniqueInteger) -> ERL_NIF_TERM;
	fn enif_is_current_process_alive(env: *mut ErlNifEnv) -> c_int;
	fn enif_is_process_alive(env: *mut ErlNifEnv, pid :*mut ErlNifPid) -> c_int;
	fn enif_is_port_alive(env: *mut ErlNifEnv, port_id: *mut ErlNifPort) -> c_int;
	fn enif_get_local_port(env: *mut ErlNifEnv, arg2: ERL_NIF_TERM, port_id: *mut ErlNifPort) -> c_int;
	fn enif_term_to_binary(env: *mut ErlNifEnv, term: ERL_NIF_TERM, bin: *mut ErlNifBinary) -> c_int;
	fn enif_binary_to_term(env: *mut ErlNifEnv, data: *const c_uchar, sz: size_t, term: *mut ERL_NIF_TERM, opts: c_uint) -> size_t;
	fn enif_port_command(env: *mut ErlNifEnv, to_port: *const ErlNifPort, msg_env: *mut ErlNifEnv, msg: ERL_NIF_TERM) -> c_int;
	fn enif_thread_type() -> c_int;
	fn enif_snprintf(buffer: *mut c_char, size: size_t, format: *const c_char, ...) -> c_int;
	fn enif_select(env: *mut ErlNifEnv, e: ErlNifEvent, flags: ErlNifSelectFlags, obj: *mut c_void, pid: *const ErlNifPid, ref_: ERL_NIF_TERM) -> c_int;
	fn enif_open_resource_type_x(arg1: *mut ErlNifEnv, name_str: *const c_char, arg2: *const ErlNifResourceTypeInit, flags: ErlNifResourceFlags, tried: *mut ErlNifResourceFlags) -> *mut ErlNifResourceType;
	fn enif_monitor_process(arg1: *mut ErlNifEnv, obj: *mut c_void, arg2: *const ErlNifPid, monitor: *mut ErlNifMonitor) -> c_int;
	fn enif_demonitor_process(arg1: *mut ErlNifEnv, obj: *mut c_void, monitor: *const ErlNifMonitor) -> c_int;
	fn enif_compare_monitors(arg1: *const ErlNifMonitor, arg2: *const ErlNifMonitor) -> c_int;
	fn enif_hash(type_: ErlNifHash, term: ERL_NIF_TERM, salt: u64) -> u64;
	fn enif_whereis_pid(env: *mut ErlNifEnv, name: ERL_NIF_TERM, pid: *mut ErlNifPid) -> c_int;
	fn enif_whereis_port(env: *mut ErlNifEnv, name: ERL_NIF_TERM, port: *mut ErlNifPort) -> c_int;
	fn enif_ioq_create(opts: ErlNifIOQueueOpts) -> *mut ErlNifIOQueue;
	fn enif_ioq_destroy(q: *mut ErlNifIOQueue);
	fn enif_ioq_enq_binary(q: *mut ErlNifIOQueue, bin: *mut ErlNifBinary, skip: size_t) -> c_int;
	fn enif_ioq_enqv(q: *mut ErlNifIOQueue, iov: *mut ErlNifIOVec, skip: size_t) -> c_int;
	fn enif_ioq_size(q: *mut ErlNifIOQueue) -> size_t;
	fn enif_ioq_deq(q: *mut ErlNifIOQueue, count: size_t, size: *mut size_t) -> c_int;
	fn enif_ioq_peek(q: *mut ErlNifIOQueue, iovlen: *mut c_int) -> *mut SysIOVec;
	fn enif_inspect_iovec(env: *mut ErlNifEnv, max_length: size_t, iovec_term: ERL_NIF_TERM, tail: *mut ERL_NIF_TERM, iovec: *mut *mut ErlNifIOVec) -> c_int;
	fn enif_free_iovec(iov: *mut ErlNifIOVec);
	fn enif_ioq_peek_head(env: *mut ErlNifEnv, q: *mut ErlNifIOQueue, size: *mut size_t, head: *mut ERL_NIF_TERM) -> c_int;
	fn enif_mutex_name(arg1: *mut ErlNifMutex) -> *mut c_char;
	fn enif_cond_name(arg1: *mut ErlNifCond) -> *mut c_char;
	fn enif_rwlock_name(arg1: *mut ErlNifRWLock) -> *mut c_char;
	fn enif_thread_name(arg1: ErlNifTid) -> *mut c_char;
	fn enif_vfprintf(); // fn enif_vfprintf(stream: *mut FILE, format: *const c_char, ap: *mut __va_list_tag) -> c_int;
	fn enif_vsnprintf(); // fn enif_vsnprintf(str: *mut c_char, size: size_t, format: *const c_char, ap: *mut __va_list_tag) -> c_int;
	fn enif_make_map_from_arrays(env: *mut ErlNifEnv, keys: *mut ERL_NIF_TERM, values: *mut ERL_NIF_TERM, cnt: size_t, map_out: *mut ERL_NIF_TERM) -> c_int;
	fn enif_select_x(env: *mut ErlNifEnv, e: ErlNifEvent, flags: ErlNifSelectFlags, obj: *mut c_void, pid: *const ErlNifPid, msg: ERL_NIF_TERM, msg_env: *mut ErlNifEnv) -> c_int;
	fn enif_make_monitor_term(env: *mut ErlNifEnv, arg1: *const ErlNifMonitor) -> ERL_NIF_TERM;
	fn enif_set_pid_undefined(pid: *mut ErlNifPid);
	fn enif_is_pid_undefined(pid: *const ErlNifPid) -> c_int;
	fn enif_term_type(env: *mut ErlNifEnv, term: ERL_NIF_TERM) -> ErlNifTermType;
}

#[cfg(unix)]
pub unsafe fn enif_get_int64(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut i64) -> c_int {
	enif_get_long(env, term, ip)
}
#[cfg(unix)]
pub unsafe fn enif_get_uint64(env: *mut ErlNifEnv, term: ERL_NIF_TERM, ip: *mut u64) -> c_int {
	enif_get_ulong(env, term, ip)
}
#[cfg(unix)]
pub unsafe fn enif_make_int64(env: *mut ErlNifEnv, i: i64) -> ERL_NIF_TERM {
	enif_make_long(env, i)
}
#[cfg(unix)]
pub unsafe fn enif_make_uint64(env: *mut ErlNifEnv, i: u64) -> ERL_NIF_TERM {
	enif_make_ulong(env, i)
}
