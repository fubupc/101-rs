use std::ffi::{c_char, CString};

// Argument forwarding
//
// see README for instructions

fn main() {
    let mut path = std::env::current_exe().unwrap();
    path.set_file_name("log");

    let executable = CString::new(path.to_str().unwrap()).unwrap();
    let args: Vec<_> = std::env::args()
        .skip(1)
        .map(|arg| CString::new(arg).unwrap())
        .collect();
    let envs: Vec<_> = std::env::vars()
        .map(|(k, v)| CString::new(format!("{k}={v}")).unwrap())
        .collect();

    let argv: Vec<_> = args
        .iter()
        .map(|arg| arg.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();
    let envp: Vec<_> = envs
        .iter()
        .map(|arg| arg.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    unsafe { libc::execve(executable.as_ptr(), argv.as_ptr(), envp.as_ptr()) };

    // if control flow ever gets here, the execve call failed.
    println!("{:#?}", std::io::Error::last_os_error());
}
