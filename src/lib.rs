extern crate winapi;

pub mod snapshot;
pub mod module;
pub mod process;
pub mod utils;

#[cfg(test)]
mod tests {
    use winapi::um::winnt::WCHAR;
    use utils::remove_nil_bytes;

    #[test]
    fn remove_nil_bytes_test() {
        // 'firefox' in utf-16 with 1 nil byte at end
        let c_string: [WCHAR; 8] = [102, 105, 114, 101, 102, 111, 120, 0];
        println!("String with nil bytes: {}", String::from_utf16(&c_string).unwrap());
        println!("String with nil bytes removed: {}", remove_nil_bytes(&c_string).unwrap())
    }
}
