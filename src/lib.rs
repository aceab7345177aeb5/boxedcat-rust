pub mod core;
pub mod services;
pub mod dns;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

/// Simple compile-time XOR obfuscation for sensitive strings
macro_rules! obf {
    ($s:expr) => {{
        const KEY: u8 = 0x5A;
        const INPUT: &[u8] = $s.as_bytes();
        const LEN: usize = INPUT.len();
        const fn xor_byte(b: u8, i: usize) -> u8 {
            b ^ KEY ^ (i as u8)
        }
        const fn obfuscate() -> [u8; LEN] {
            let mut out = [0u8; LEN];
            let mut i = 0;
            while i < LEN {
                out[i] = xor_byte(INPUT[i], i);
                i += 1;
            }
            out
        }
        const OBF: [u8; LEN] = obfuscate();
        let mut result = String::with_capacity(LEN);
        let mut i = 0;
        while i < LEN {
            result.push((OBF[i] ^ KEY ^ (i as u8)) as char);
            i += 1;
        }
        result
    }};
}

fn safe_str(ptr: *const c_char) -> String {
    if ptr.is_null() { return String::new(); }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() }
}

fn sanitize_name(name: &str) -> bool {
    !name.contains('\\') && !name.contains('/') && !name.contains("..") && !name.contains('\0') && !name.is_empty()
}

/// Get the server directory. Priority:
/// 1. BOXEDCAT_SERVER_DIR env var
/// 2. Directory where boxedcat.exe is located
fn get_server_dir() -> String {
    if let Ok(dir) = std::env::var("BOXEDCAT_SERVER_DIR") {
        return dir;
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
        .unwrap_or_else(|| "C:\\".to_string())
}

struct GlobalState {
    services: Vec<BcServiceInfo>,
    ports: Vec<BcPortInfo>,
}

unsafe impl Send for GlobalState {}
unsafe impl Sync for GlobalState {}

static STATE: Mutex<Option<Box<GlobalState>>> = Mutex::new(None);

#[repr(C)]
pub struct BcServiceInfo {
    pub name: *const c_char,
    pub display_name: *const c_char,
    pub port: u16,
    pub status: *const c_char,
    pub version: *const c_char,
    pub config_path: *const c_char,
}

#[repr(C)]
pub struct BcPortInfo {
    pub service: *const c_char,
    pub primary: u16,
    pub fallbacks: *const c_char,
}

#[repr(C)]
pub struct BcServiceList {
    pub items: *const BcServiceInfo,
    pub count: usize,
}

#[repr(C)]
pub struct BcPortStatus {
    pub port: u16,
    pub in_use: bool,
    pub process_name: *const c_char,
    pub pid: u32,
}

#[repr(C)]
pub struct BcPortStatusList {
    pub items: *const BcPortStatus,
    pub count: usize,
}

#[no_mangle]
pub extern "C" fn boxedcat_init() {
    let mut services = Vec::new();
    let mut ports = Vec::new();

    let xampp = get_server_dir();

    let cs = |s: &str| -> *const c_char { CString::new(s).unwrap_or_default().into_raw() };

    let apache_conf = format!("{}\\apache\\conf\\httpd.conf", xampp);
    let installed = std::path::Path::new(&apache_conf).exists();
    services.push(BcServiceInfo {
        name: cs("apache"),
        display_name: cs("Apache HTTP Server"),
        port: 80,
        status: cs(if installed { "Detected" } else { "Not Found" }),
        version: cs("2.4.66"),
        config_path: cs(&apache_conf),
    });

    let mariadb_exe = format!("{}\\mysql\\bin\\mysqld.exe", xampp);
    let installed2 = std::path::Path::new(&mariadb_exe).exists();
    services.push(BcServiceInfo {
        name: cs("mariadb"),
        display_name: cs("MariaDB"),
        port: 3306,
        status: cs(if installed2 { "Detected" } else { "Not Found" }),
        version: cs("11.4.13"),
        config_path: cs(&format!("{}\\mysql\\bin", xampp)),
    });

    let php_ini = format!("{}\\php\\php.ini", xampp);
    let installed3 = std::path::Path::new(&php_ini).exists();
    services.push(BcServiceInfo {
        name: cs("php"),
        display_name: cs("PHP"),
        port: 9000,
        status: cs(if installed3 { "Detected" } else { "Not Found" }),
        version: cs("8.5.10"),
        config_path: cs(&format!("{}\\php", xampp)),
    });

    let certs_dir = format!("{}\\certs", xampp);
    let cert_exists = std::path::Path::new(&format!("{}\\server.crt", certs_dir)).exists();
    let key_exists = std::path::Path::new(&format!("{}\\server.key", certs_dir)).exists();
    let tls_configured = cert_exists && key_exists;
    services.push(BcServiceInfo {
        name: cs("https"),
        display_name: cs("HTTPS/TLS"),
        port: 443,
        status: cs(if tls_configured { "Configured" } else { "No Certificate" }),
        version: cs("TLS 1.2/1.3"),
        config_path: cs(&certs_dir),
    });

    ports.push(BcPortInfo { service: cs("http"), primary: 80, fallbacks: cs("8080, 8000, 8088") });
    ports.push(BcPortInfo { service: cs("https"), primary: 443, fallbacks: cs("4433, 4443, 8443") });
    ports.push(BcPortInfo { service: cs("mariadb"), primary: 3306, fallbacks: cs("3307, 3308, 3309") });
    ports.push(BcPortInfo { service: cs("postgresql"), primary: 5432, fallbacks: cs("5433, 5434, 5435") });
    ports.push(BcPortInfo { service: cs("redis"), primary: 6379, fallbacks: cs("6380, 6381, 6382") });

    *STATE.lock().unwrap() = Some(Box::new(GlobalState { services, ports }));
}

#[no_mangle]
pub extern "C" fn boxedcat_get_services() -> BcServiceList {
    let lock = STATE.lock().unwrap();
    if let Some(ref state) = *lock {
        BcServiceList { items: state.services.as_ptr(), count: state.services.len() }
    } else {
        BcServiceList { items: std::ptr::null(), count: 0 }
    }
}

#[no_mangle]
pub extern "C" fn boxedcat_get_port_count() -> usize {
    let lock = STATE.lock().unwrap();
    lock.as_ref().map_or(0, |s| s.ports.len())
}

#[no_mangle]
pub extern "C" fn boxedcat_get_port(index: usize) -> *const BcPortInfo {
    let lock = STATE.lock().unwrap();
    if let Some(ref state) = *lock {
        if index < state.ports.len() {
            return &state.ports[index] as *const BcPortInfo;
        }
    }
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn boxedcat_get_version() -> *const c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn boxedcat_service_start(name: *const c_char) -> bool {
    let name = safe_str(name);
    if name.is_empty() { return false; }
    let xampp = get_server_dir();
    let root = std::path::PathBuf::from(&xampp);

    let result = match name.as_str() {
        "apache" => {
            let exe = root.join("apache").join("bin").join("httpd.exe");
            if !exe.exists() { return false; }
            std::process::Command::new(&exe)
                .args(["-k", "start"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
        }
        "mariadb" => {
            let exe = root.join("mysql").join("bin").join("mysqld.exe");
            let ini = root.join("mysql").join("my.ini");
            if !exe.exists() { return false; }
            std::process::Command::new(&exe)
                .args(["--defaults-file", &ini.to_string_lossy()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
        }
        _ => return false,
    };
    result.map(|s| s.success()).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn boxedcat_service_stop(name: *const c_char) -> bool {
    let name = safe_str(name);
    if name.is_empty() { return false; }
    let xampp = get_server_dir();
    let root = std::path::PathBuf::from(&xampp);

    let result = match name.as_str() {
        "apache" => {
            let exe = root.join("apache").join("bin").join("httpd.exe");
            if !exe.exists() { return false; }
            std::process::Command::new(&exe)
                .args(["-k", "stop"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
        }
        "mariadb" => {
            let mysql = root.join("mysql").join("bin").join("mysql.exe");
            if !mysql.exists() { return false; }
            std::process::Command::new(&mysql)
                .args(["-u", "root", "--password=", "-e", "SHUTDOWN;"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
        }
        _ => return false,
    };
    result.map(|s| s.success()).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn boxedcat_service_is_running(name: *const c_char) -> bool {
    let name = safe_str(name);
    if name.is_empty() { return false; }
    let exe_name = match name.as_str() {
        "apache" => "httpd.exe",
        "mariadb" => "mysqld.exe",
        _ => return false,
    };
    find_process_by_name(exe_name)
}

#[no_mangle]
pub extern "C" fn boxedcat_refresh_service(name: *const c_char) -> *const c_char {
    let running = boxedcat_service_is_running(name);
    CString::new(if running { "Running" } else { "Stopped" }).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn boxedcat_scan_port(port: u16) -> BcPortStatus {
    let mut status = BcPortStatus {
        port,
        in_use: false,
        process_name: CString::new("").unwrap().into_raw(),
        pid: 0,
    };

    let mut tcp_table_len: u32 = 0;

    unsafe {
        let _ = windows_sys::Win32::NetworkManagement::IpHelper::GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut tcp_table_len,
            windows_sys::Win32::Foundation::TRUE,
            windows_sys::Win32::Networking::WinSock::AF_INET as u32,
            windows_sys::Win32::NetworkManagement::IpHelper::TCP_TABLE_OWNER_PID_LISTENER,
            0,
        );

        if tcp_table_len > 0 {
            let mut buf: Vec<u8> = vec![0u8; tcp_table_len as usize];
            let result = windows_sys::Win32::NetworkManagement::IpHelper::GetExtendedTcpTable(
                buf.as_mut_ptr() as *mut _,
                &mut tcp_table_len,
                windows_sys::Win32::Foundation::TRUE,
                windows_sys::Win32::Networking::WinSock::AF_INET as u32,
                windows_sys::Win32::NetworkManagement::IpHelper::TCP_TABLE_OWNER_PID_LISTENER,
                0,
            );

            if result == 0 {
                let num_entries = *(buf.as_ptr() as *const u32);
                let entry_size = std::mem::size_of::< windows_sys::Win32::NetworkManagement::IpHelper::MIB_TCPROW_OWNER_PID>();
                for i in 0..num_entries {
                    let offset = 4 + (i as usize) * entry_size;
                    if offset + entry_size <= buf.len() {
                        let row = &*(buf[offset..].as_ptr() as *const windows_sys::Win32::NetworkManagement::IpHelper::MIB_TCPROW_OWNER_PID);
                        let row_port = ((row.dwLocalPort & 0xFF) << 8) | ((row.dwLocalPort >> 8) & 0xFF);
                        let row_port = row_port as u16;
                        if row_port == port {
                            status.in_use = true;
                            status.pid = row.dwOwningPid;
                            break;
                        }
                    }
                }
            }
        }
    }

    if status.in_use && status.pid > 0 {
        status.process_name = get_process_name(status.pid);
    }

    status
}

fn get_process_name(pid: u32) -> *const c_char {
    unsafe {
        let proc = windows_sys::Win32::System::Threading::OpenProcess(
            windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION,
            windows_sys::Win32::Foundation::FALSE,
            pid,
        );
        if proc.is_null() {
            return CString::new("unknown").unwrap().into_raw();
        }
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let ok = windows_sys::Win32::System::Threading::QueryFullProcessImageNameW(
            proc,
            0,
            buf.as_mut_ptr(),
            &mut size,
        );
        windows_sys::Win32::Foundation::CloseHandle(proc);
        if ok != 0 {
            let full_path = String::from_utf16_lossy(&buf[..size as usize]);
            let name = std::path::Path::new(&full_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            CString::new(name).unwrap().into_raw()
        } else {
            CString::new("unknown").unwrap().into_raw()
        }
    }
}

fn find_process_by_name(exe_name: &str) -> bool {
    unsafe {
        let snap = windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS,
            0,
        );
        if snap == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE { return false; }

        let mut entry: windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W>() as u32;

        let mut found = false;
        if windows_sys::Win32::System::Diagnostics::ToolHelp::Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let name = String::from_utf16_lossy(&entry.szExeFile);
                if name.eq_ignore_ascii_case(exe_name) {
                    found = true;
                    break;
                }
                if windows_sys::Win32::System::Diagnostics::ToolHelp::Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        windows_sys::Win32::Foundation::CloseHandle(snap);
        found
    }
}

static SCAN_PORTS: [u16; 10] = [80, 443, 3306, 5432, 6379, 8080, 8443, 3307, 3308, 3309];

#[no_mangle]
pub extern "C" fn boxedcat_scan_port_count() -> usize {
    SCAN_PORTS.len()
}

#[no_mangle]
pub extern "C" fn boxedcat_scan_port_get(index: usize) -> BcPortStatus {
    if index >= SCAN_PORTS.len() {
        return BcPortStatus { port: 0, in_use: false, process_name: CString::new("").unwrap().into_raw(), pid: 0 };
    }
    boxedcat_scan_port(SCAN_PORTS[index])
}

#[no_mangle]
pub extern "C" fn boxedcat_free_port_status(status: BcPortStatus) {
    unsafe {
        drop(CString::from_raw(status.process_name as *mut c_char));
    }
}

#[repr(C)]
pub struct BcProject {
    pub name: *const c_char,
    pub path: *const c_char,
    pub server_name: *const c_char,
    pub has_entry_point: bool,
    pub project_type: *const c_char,
}

#[repr(C)]
pub struct BcProjectList {
    pub items: *const BcProject,
    pub count: usize,
}

#[no_mangle]
pub extern "C" fn boxedcat_scan_projects() -> BcProjectList {
    let xampp = get_server_dir();
    let www = std::path::PathBuf::from(&xampp).join("www");
    let mut projects = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&www) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if name.starts_with('.') || name == "dashboard" || name == "img" || name == "css" || name == "js" || name == "webalizer" { continue; }

            let has_index_php = path.join("index.php").exists();
            let has_index_html = path.join("index.html").exists();
            let has_composer = path.join("composer.json").exists();
            let has_package = path.join("package.json").exists();
            let has_entry = has_index_php || has_index_html;

            let ptype = if has_composer { "PHP" }
                else if has_package { "Node.js" }
                else if has_index_php { "PHP" }
                else if has_index_html { "Static" }
                else { "Unknown" };

            projects.push(BcProject {
                name: CString::new(name.clone()).unwrap().into_raw(),
                path: CString::new(path.to_string_lossy().to_string()).unwrap().into_raw(),
                server_name: CString::new(format!("{}.test", name)).unwrap().into_raw(),
                has_entry_point: has_entry,
                project_type: CString::new(ptype).unwrap().into_raw(),
            });
        }
    }

    let count = projects.len();
    let boxed = projects.into_boxed_slice();
    let ptr = Box::into_raw(boxed) as *const BcProject;
    BcProjectList { items: ptr, count }
}

#[no_mangle]
pub extern "C" fn boxedcat_free_projects(list: BcProjectList) {
    unsafe {
        let slice = std::slice::from_raw_parts(list.items, list.count);
        let owned = Box::from_raw(slice as *const [BcProject] as *mut [BcProject]);
        for p in owned.iter() {
            drop(CString::from_raw(p.name as *mut c_char));
            drop(CString::from_raw(p.path as *mut c_char));
            drop(CString::from_raw(p.server_name as *mut c_char));
            drop(CString::from_raw(p.project_type as *mut c_char));
        }
    }
}

#[no_mangle]
pub extern "C" fn boxedcat_create_vhost(name: *const c_char, port: u16) -> bool {
    let name = safe_str(name);
    if !sanitize_name(&name) { return false; }
    let xampp = get_server_dir();
    let www = std::path::PathBuf::from(&xampp).join("www").join(&name);

    if !www.exists() { return false; }

    let server_name = format!("{}.test", name);
    let doc_root = www.to_string_lossy().replace('\\', "/");
    let vhost_conf = format!(
        "<VirtualHost *:{}>\n    DocumentRoot \"{}\"\n    ServerName {}\n    <Directory \"{}\">\n        Options Indexes FollowSymLinks\n        AllowOverride All\n        Require all granted\n    </Directory>\n</VirtualHost>",
        port, doc_root, server_name, doc_root
    );

    let vhosts_dir = std::path::PathBuf::from(&xampp).join("apache").join("conf").join("extra");
    let _ = std::fs::create_dir_all(&vhosts_dir);
    let vhost_file = vhosts_dir.join(format!("boxedcat-{}.conf", name));
    std::fs::write(&vhost_file, vhost_conf).is_ok()
}

#[no_mangle]
pub extern "C" fn boxedcat_delete_vhost(name: *const c_char) -> bool {
    let name = safe_str(name);
    if !sanitize_name(&name) { return false; }
    let xampp = get_server_dir();
    let vhost_file = std::path::PathBuf::from(&xampp)
        .join("apache").join("conf").join("extra")
        .join(format!("boxedcat-{}.conf", name));
    if vhost_file.exists() {
        std::fs::remove_file(&vhost_file).is_ok()
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn boxedcat_dns_start(port: u16, upstream: *const c_char) -> bool {
    let upstream = safe_str(upstream);
    if upstream.is_empty() { return false; }
    dns::start_dns_proxy(port, &upstream)
}

#[no_mangle]
pub extern "C" fn boxedcat_dns_stop() {
    dns::stop_dns_proxy();
}

#[no_mangle]
pub extern "C" fn boxedcat_dns_is_running() -> bool {
    dns::is_dns_running()
}

#[no_mangle]
pub extern "C" fn boxedcat_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

/// Check if an adapter name or description matches known virtual/hypervisor patterns.
fn is_virtual_adapter(friendly_name: &str, description: &str) -> bool {
    let combined = format!("{} {}", friendly_name, description).to_lowercase();
    let skip_patterns = [
        "vpn", "virtual", "docker", "vethernet", "hyper-v", "hyper",
        "wsl", "vmware", "virtualbox", "npcap", "loopback", "teredo",
        "isatap", "6to4", "wifi direct", "bluetooth", "cellular",
        "hamachi", "tunnel", "tap", "openvpn", "wireguard", "tailscale",
        "juniper", "fortinet", "cisco anyconnect", "sonicwall",
        "ras asynchronous", "ras server", "ppp", "serial",
        "host-only", "nat network", "bridged adapter",
    ];
    skip_patterns.iter().any(|p| combined.contains(p))
}

/// Get local network IP from physical network adapter.
/// Filters out virtual adapters (VPN, Docker, WSL, Hyper-V, etc.)
#[no_mangle]
pub extern "C" fn boxedcat_get_local_ip() -> *const c_char {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST,
        GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_DNS_SERVER, IP_ADAPTER_ADDRESSES_LH,
    };
    use windows_sys::Win32::Networking::WinSock::AF_INET;

    let mut result_ip = CString::new("127.0.0.1").unwrap().into_raw();

    unsafe {
        let mut buffer_len: u32 = 15000;
        let mut buffer: Vec<u8> = vec![0u8; buffer_len as usize];

        let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

        let ret = GetAdaptersAddresses(
            AF_INET as u32,
            flags,
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH,
            &mut buffer_len,
        );

        if ret == 0 {
            let mut adapter = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
            while !adapter.is_null() {
                let a = &*adapter;

                if a.OperStatus != 1 {
                    adapter = a.Next;
                    continue;
                }

                if a.IfType == 24 {
                    adapter = a.Next;
                    continue;
                }

                // Get friendly name
                let name_len = (0..256).find(|&i| *(*a).FriendlyName.add(i) == 0).unwrap_or(256);
                let friendly_name = String::from_utf16_lossy(
                    std::slice::from_raw_parts((*a).FriendlyName, name_len)
                );

                // Get description
                let desc_len = (0..256).find(|&i| *(*a).Description.add(i) == 0).unwrap_or(256);
                let description = String::from_utf16_lossy(
                    std::slice::from_raw_parts((*a).Description, desc_len)
                );

                if is_virtual_adapter(&friendly_name, &description) {
                    adapter = a.Next;
                    continue;
                }

                if !a.FirstUnicastAddress.is_null() {
                    let unicast = &*a.FirstUnicastAddress;
                    if !unicast.Address.lpSockaddr.is_null() {
                        let sockaddr = &*(unicast.Address.lpSockaddr as *const windows_sys::Win32::Networking::WinSock::SOCKADDR_IN);
                        let octets = sockaddr.sin_addr.S_un.S_un_b;

                        if (octets.s_b1 == 0 && octets.s_b2 == 0 && octets.s_b3 == 0 && octets.s_b4 == 0)
                            || octets.s_b1 == 127
                            || (octets.s_b1 == 169 && octets.s_b2 == 254)
                        {
                            adapter = a.Next;
                            continue;
                        }

                        let ip = format!("{}.{}.{}.{}", octets.s_b1, octets.s_b2, octets.s_b3, octets.s_b4);
                        drop(CString::from_raw(result_ip));
                        result_ip = CString::new(ip).unwrap().into_raw();
                        break;
                    }
                }

                adapter = a.Next;
            }
        }
    }

    result_ip
}

/// Get adapter name (friendly name of the network interface)
#[no_mangle]
pub extern "C" fn boxedcat_get_adapter_name() -> *const c_char {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST,
        GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_DNS_SERVER, IP_ADAPTER_ADDRESSES_LH,
    };
    use windows_sys::Win32::Networking::WinSock::AF_INET;

    let mut result_name = CString::new("Unknown").unwrap().into_raw();

    unsafe {
        let mut buffer_len: u32 = 15000;
        let mut buffer: Vec<u8> = vec![0u8; buffer_len as usize];

        let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

        let ret = GetAdaptersAddresses(
            AF_INET as u32,
            flags,
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH,
            &mut buffer_len,
        );

        if ret == 0 {
            let mut adapter = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
            while !adapter.is_null() {
                let a = &*adapter;

                if a.OperStatus != 1 {
                    adapter = a.Next;
                    continue;
                }
                if a.IfType == 24 {
                    adapter = a.Next;
                    continue;
                }

                let name_len = (0..256).find(|&i| *(*a).FriendlyName.add(i) == 0).unwrap_or(256);
                let friendly_name = String::from_utf16_lossy(
                    std::slice::from_raw_parts((*a).FriendlyName, name_len)
                );

                let desc_len = (0..256).find(|&i| *(*a).Description.add(i) == 0).unwrap_or(256);
                let description = String::from_utf16_lossy(
                    std::slice::from_raw_parts((*a).Description, desc_len)
                );

                if is_virtual_adapter(&friendly_name, &description) {
                    adapter = a.Next;
                    continue;
                }

                if !a.FirstUnicastAddress.is_null() {
                    let unicast = &*a.FirstUnicastAddress;
                    if !unicast.Address.lpSockaddr.is_null() {
                        let sockaddr = &*(unicast.Address.lpSockaddr as *const windows_sys::Win32::Networking::WinSock::SOCKADDR_IN);
                        let octets = sockaddr.sin_addr.S_un.S_un_b;

                        if (octets.s_b1 == 0 && octets.s_b2 == 0 && octets.s_b3 == 0 && octets.s_b4 == 0)
                            || octets.s_b1 == 127
                            || (octets.s_b1 == 169 && octets.s_b2 == 254)
                        {
                            adapter = a.Next;
                            continue;
                        }

                        drop(CString::from_raw(result_name));
                        result_name = CString::new(friendly_name).unwrap().into_raw();
                        break;
                    }
                }

                adapter = a.Next;
            }
        }
    }

    result_name
}

/// Get workstation hostname
#[no_mangle]
pub extern "C" fn boxedcat_get_hostname() -> *const c_char {
    match std::env::var("COMPUTERNAME") {
        Ok(name) => CString::new(name).unwrap().into_raw(),
        Err(_) => CString::new("Unknown").unwrap().into_raw(),
    }
}

/// Get total physical RAM in MB
#[no_mangle]
pub extern "C" fn boxedcat_get_total_ram_mb() -> u64 {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    unsafe {
        let mut mem_info: MEMORYSTATUSEX = std::mem::zeroed();
        mem_info.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        let ok = GlobalMemoryStatusEx(&mut mem_info);
        if ok != 0 {
            mem_info.ullTotalPhys / (1024 * 1024)
        } else {
            0
        }
    }
}

/// Get motherboard name via WMI (simplified - returns registry info)
#[no_mangle]
pub extern "C" fn boxedcat_get_motherboard() -> *const c_char {
    // Read from registry as fallback (WMI would need COM initialization)
    use windows_sys::Win32::System::Registry::{
        RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_LOCAL_MACHINE,
        KEY_READ, REG_SZ,
    };

    let subkey: Vec<u16> = "SYSTEM\\CurrentControlSet\\Control\\SystemInformation"
        .encode_utf16().chain(std::iter::once(0)).collect();
    let val_name: Vec<u16> = "SystemProductName".encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut hkey = std::ptr::null_mut();
        let ret = RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut hkey);
        if ret == 0 {
            let mut buf = [0u16; 256];
            let mut buf_size = (buf.len() * 2) as u32;
            let mut reg_type = REG_SZ;
            let ret2 = RegQueryValueExW(
                hkey, val_name.as_ptr(), std::ptr::null_mut(),
                &mut reg_type, buf.as_mut_ptr() as *mut u8, &mut buf_size,
            );
            RegCloseKey(hkey);
            if ret2 == 0 {
                let len = (buf_size / 2).saturating_sub(1) as usize;
                let name = String::from_utf16_lossy(&buf[..len]);
                if !name.is_empty() {
                    return CString::new(name).unwrap().into_raw();
                }
            }
        }
    }

    CString::new("Unknown").unwrap().into_raw()
}

/// Download a file from URL to a local path using WinHTTP.
/// Returns true on success, false on failure.
#[no_mangle]
pub extern "C" fn boxedcat_download_file(url: *const c_char, dest_path: *const c_char) -> bool {
    use windows_sys::Win32::Networking::WinHttp::{
        WinHttpOpen, WinHttpConnect, WinHttpOpenRequest, WinHttpSendRequest,
        WinHttpReceiveResponse, WinHttpReadData, WinHttpCloseHandle,
        WINHTTP_FLAG_BYPASS_PROXY_CACHE,
    };

    let url_str = safe_str(url);
    let dest_str = safe_str(dest_path);
    if url_str.is_empty() || dest_str.is_empty() { return false; }

    let url_lower = url_str.to_lowercase();
    let use_https = url_lower.starts_with("https://");
    let default_port: u16 = if use_https { 443 } else { 80 };

    let without_scheme = if use_https {
        url_str.strip_prefix("https://").unwrap_or(&url_str)
    } else {
        url_str.strip_prefix("http://").unwrap_or(&url_str)
    };
    let slash_pos = without_scheme.find('/').unwrap_or(without_scheme.len());
    let host = &without_scheme[..slash_pos];
    let path = if slash_pos < without_scheme.len() {
        &without_scheme[slash_pos..]
    } else {
        "/"
    };

    unsafe {
        let agent = WinHttpOpen(
            b"BoxedCat\0".as_ptr() as *const u16,
            0,
            std::ptr::null(),
            std::ptr::null(),
            0,
        );
        if agent.is_null() { return false; }

        let host_wide: Vec<u16> = host.encode_utf16().chain(std::iter::once(0)).collect();
        let connect = WinHttpConnect(agent, host_wide.as_ptr(), default_port, 0);
        if connect.is_null() {
            WinHttpCloseHandle(agent);
            return false;
        }

        let verb = b"GET\0";
        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let accept = b"*/*\0";
        let request = WinHttpOpenRequest(
            connect,
            verb.as_ptr() as *const u16,
            path_wide.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            accept.as_ptr() as *const *const u16,
            WINHTTP_FLAG_BYPASS_PROXY_CACHE,
        );
        if request.is_null() {
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(agent);
            return false;
        }

        let result = WinHttpSendRequest(
            request,
            std::ptr::null(),
            0,
            std::ptr::null(),
            0,
            0,
            0,
        );

        let mut success = false;
        if result != 0 {
            if WinHttpReceiveResponse(request, std::ptr::null_mut()) != 0 {
                let mut bytes_available: u32 = 0;
                let mut total_read: u32 = 0;

                let mut file_buf: Vec<u8> = Vec::new();
                loop {
                    bytes_available = 0;
                    if windows_sys::Win32::Networking::WinHttp::WinHttpQueryDataAvailable(request, &mut bytes_available) == 0 {
                        break;
                    }
                    if bytes_available == 0 { break; }

                    let mut read_buf = vec![0u8; bytes_available as usize];
                    let mut bytes_read: u32 = 0;
                    if WinHttpReadData(request, read_buf.as_mut_ptr() as *mut _, bytes_available, &mut bytes_read) == 0 {
                        break;
                    }
                    file_buf.extend_from_slice(&read_buf[..bytes_read as usize]);
                    total_read += bytes_read;
                }

                if total_read > 0 {
                    success = std::fs::write(&dest_str, &file_buf).is_ok();
                }
            }
        }

        WinHttpCloseHandle(request);
        WinHttpCloseHandle(connect);
        WinHttpCloseHandle(agent);
        success
    }
}

/// Show a native save dialog and return the selected path (or empty on cancel)
#[no_mangle]
pub extern "C" fn boxedcat_save_dialog(default_name: *const c_char) -> *const c_char {
    use windows_sys::Win32::UI::Controls::Dialogs::{GetSaveFileNameW, OPENFILENAMEW};

    let name = safe_str(default_name);
    let mut filename: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    filename.resize(260, 0);

    let filter: Vec<u16> = "All Files\0*.*\0Zip Archives\0*.zip\0\0".encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.lpstrFile = filename.as_mut_ptr();
        ofn.nMaxFile = filename.len() as u32;
        ofn.Flags = 0x00080000 | 0x00001000;

        if GetSaveFileNameW(&mut ofn) != 0 {
            let result = String::from_utf16_lossy(
                &filename[..filename.iter().position(|&c| c == 0).unwrap_or(filename.len())]
            );
            CString::new(result).unwrap().into_raw()
        } else {
            CString::new("").unwrap().into_raw()
        }
    }
}

/// Show a native folder browse dialog and return the selected path
#[no_mangle]
pub extern "C" fn boxedcat_browse_folder(title: *const c_char) -> *const c_char {
    use windows_sys::Win32::UI::Shell::{SHBrowseForFolderW, SHGetPathFromIDListW, BROWSEINFOW};

    let title_str = safe_str(title);
    let title_wide: Vec<u16> = title_str.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut bi: BROWSEINFOW = std::mem::zeroed();
        bi.lpszTitle = title_wide.as_ptr();
        bi.ulFlags = 0x00000040;

        let pidl = SHBrowseForFolderW(&bi);
        if !pidl.is_null() {
            let mut path_buf = [0u16; 260];
            if SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr()) != 0 {
                windows_sys::Win32::System::Com::CoTaskMemFree(pidl as *const _);
                let path = String::from_utf16_lossy(
                    &path_buf[..path_buf.iter().position(|&c| c == 0).unwrap_or(path_buf.len())]
                );
                return CString::new(path).unwrap().into_raw();
            }
            windows_sys::Win32::System::Com::CoTaskMemFree(pidl as *const _);
        }
    }

    CString::new("").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn boxedcat_shutdown() {
    if let Some(state) = STATE.lock().unwrap().take() {
        for s in state.services {
            unsafe {
                drop(CString::from_raw(s.name as *mut c_char));
                drop(CString::from_raw(s.display_name as *mut c_char));
                drop(CString::from_raw(s.status as *mut c_char));
                drop(CString::from_raw(s.version as *mut c_char));
                drop(CString::from_raw(s.config_path as *mut c_char));
            }
        }
        for p in state.ports {
            unsafe {
                drop(CString::from_raw(p.service as *mut c_char));
                drop(CString::from_raw(p.fallbacks as *mut c_char));
            }
        }
    }
}

/// TLS Certificate management
fn get_certs_dir() -> std::path::PathBuf {
    let xampp = get_server_dir();
    std::path::PathBuf::from(xampp).join("certs")
}

/// Generate a self-signed TLS certificate for localhost development.
/// Returns true on success, false on failure.
#[no_mangle]
pub extern "C" fn boxedcat_generate_cert() -> bool {
    let certs_dir = get_certs_dir();
    if std::fs::create_dir_all(&certs_dir).is_err() {
        return false;
    }

    let cert_path = certs_dir.join("server.crt");
    let key_path = certs_dir.join("server.key");

    // Don't regenerate if already exists
    if cert_path.exists() && key_path.exists() {
        return true;
    }

    let mut params = rcgen::CertificateParams::new(vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ]).unwrap();
    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        rcgen::DnValue::Utf8String("BoxedCat Dev Server".to_string()),
    );
    params.not_before = time::OffsetDateTime::now_utc();
    params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);

    let key_pair = match rcgen::KeyPair::generate() {
        Ok(kp) => kp,
        Err(_) => return false,
    };

    let cert = match params.self_signed(&key_pair) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    if std::fs::write(&cert_path, &cert_pem).is_err() {
        return false;
    }
    if std::fs::write(&key_path, &key_pem).is_err() {
        return false;
    }

    true
}

/// Check if TLS certificates are configured.
/// Returns true if both server.crt and server.key exist.
#[no_mangle]
pub extern "C" fn boxedcat_tls_is_configured() -> bool {
    let certs_dir = get_certs_dir();
    certs_dir.join("server.crt").exists() && certs_dir.join("server.key").exists()
}

/// Get the path to the TLS certificates directory.
#[no_mangle]
pub extern "C" fn boxedcat_get_certs_dir() -> *const c_char {
    let dir = get_certs_dir();
    CString::new(dir.to_string_lossy().to_string()).unwrap().into_raw()
}

/// Delete TLS certificates (regenerate on next call to boxedcat_generate_cert).
#[no_mangle]
pub extern "C" fn boxedcat_delete_cert() -> bool {
    let certs_dir = get_certs_dir();
    let _ = std::fs::remove_file(certs_dir.join("server.crt"));
    let _ = std::fs::remove_file(certs_dir.join("server.key"));
    true
}
