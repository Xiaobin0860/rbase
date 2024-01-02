use sysinfo::{CpuRefreshKind, System};

lazy_static::lazy_static! {
    static ref SYS_INFO: SysInfo = {
        let mut sys = System::new();
        sys.refresh_cpu_specifics(CpuRefreshKind::new());
        SysInfo{
            cpu_count: sys.cpus().len() as u32,
            cpu_name: sys.global_cpu_info().brand().to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    };
}

/// 系统静态信息
#[derive(Debug)]
pub struct SysInfo {
    /// 逻辑核数
    pub cpu_count: u32,
    /// cpu型号
    pub cpu_name: String,
    /// 操作系统 linux/macos/windows/...
    pub os: String,
    /// 系统架构 x86_64/arm/...
    pub arch: String,
}

/// 获取系统静态信息
pub fn get_sys_info() -> &'static SysInfo {
    &SYS_INFO
}

/// 获取1分钟内的系统负载
pub fn get_sys_load() -> f32 {
    System::load_average().one as f32
}

/// 获取可用内存MB
pub fn get_available_memory() -> u32 {
    let mut sys = System::new();
    sys.refresh_memory();
    (sys.available_memory() / 1024 / 1024) as u32
}
