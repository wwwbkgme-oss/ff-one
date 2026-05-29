#[repr(C)] pub struct FfPluginCtx{pub host:*const std::ffi::c_void,pub load_tick:u64}
#[repr(C)] pub struct FfPluginInfo{pub id:*const std::ffi::c_char,pub version:*const std::ffi::c_char,pub name:*const std::ffi::c_char,pub abi_version:u32}
unsafe impl Send for FfPluginInfo{}
unsafe impl Sync for FfPluginInfo{}
pub const FF_ABI_VERSION:u32=1;
pub type InfoFn=unsafe extern "C" fn()->*const FfPluginInfo;
pub type InitFn=unsafe extern "C" fn(ctx:*const FfPluginCtx)->i32;
pub type TickFn=unsafe extern "C" fn(tick:u64)->i32;
pub type ShutdownFn=unsafe extern "C" fn()->i32;
#[macro_export]
macro_rules! export_plugin {
    (id:$id:literal,version:$v:literal,name:$n:literal,init:$init:ident,tick:$tick:ident,shutdown:$sd:ident $(,)?)=>{
        static _FF_INFO:$crate::abi::FfPluginInfo=$crate::abi::FfPluginInfo{
            id:concat!($id,"\0").as_ptr()as*const::std::ffi::c_char,
            version:concat!($v,"\0").as_ptr()as*const::std::ffi::c_char,
            name:concat!($n,"\0").as_ptr()as*const::std::ffi::c_char,
            abi_version:$crate::abi::FF_ABI_VERSION,
        };
        #[no_mangle] pub extern "C" fn ff_plugin_info()->*const $crate::abi::FfPluginInfo{&_FF_INFO}
        #[no_mangle] pub extern "C" fn ff_plugin_init(ctx:*const $crate::abi::FfPluginCtx)->i32{$init(ctx)}
        #[no_mangle] pub extern "C" fn ff_plugin_tick(tick:u64)->i32{$tick(tick)}
        #[no_mangle] pub extern "C" fn ff_plugin_shutdown()->i32{$sd()}
    };
}
