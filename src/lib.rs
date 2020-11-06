extern crate winapi;

pub mod snapshot;
pub mod module;
pub mod process;
pub mod utils;

#[cfg(test)]
mod tests {
    use process::Process;

    /// Find the 'firefox' process
    fn firefox() -> Process {
        Process::find("firefox.exe")
            .expect("Could not find process 'firefox.exe'")
    }

    /// Print the PID of firefox
    #[test]
    fn get_firefox_pid() {
        println!("Firefox PID = {}", firefox().pid())
    }

    /// Find and print the address of the DirectX11 DLL in firefox
    #[test]
    fn find_directx_11_module_firefox() {
        println!("Module Address = {}", firefox()
            .find_module("d3d11.dll")
            .expect("Could not find the 'd3d11.dll' module in the 'firefox.exe' process")
            .address())
    }
}