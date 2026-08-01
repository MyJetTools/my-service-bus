pub fn format_bytes(bytes: i64) -> String {
    let v = bytes as f64;
    if v < 1024.0 {
        return format!("{} Bytes", bytes);
    }

    let kb = v / 1024.0;
    if kb < 1024.0 {
        return format!("{:.2} KBytes", kb);
    }

    let mb = kb / 1024.0;
    if mb < 1024.0 {
        return format!("{:.2} MBytes", mb);
    }

    let gb = mb / 1024.0;
    format!("{:.2} GBytes", gb)
}

pub fn format_bytes_per_sec(bytes_per_sec: i64) -> String {
    format!("{} per sec", format_bytes(bytes_per_sec))
}

pub fn format_unix_micros(unix_micros: i64) -> String {
    let ms = (unix_micros / 1000) as f64;
    let date = js_sys::Date::new(&ms.into());
    let iso: String = date.to_iso_string().into();
    iso
}
