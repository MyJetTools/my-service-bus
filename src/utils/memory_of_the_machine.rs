/// The total and the used memory of the machine, in bytes, for the status page and the MCP
/// overview. sysinfo 0.40 answers a `Result` where it can not read the machine: that reports
/// zeroes rather than failing the page.
pub fn memory_of_the_machine() -> (u64, u64) {
    match sysinfo::System::new_all() {
        Ok(mut sys_info) => {
            sys_info.refresh_all();
            (sys_info.total_memory(), sys_info.used_memory())
        }
        Err(_) => (0, 0),
    }
}
