use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::os::raw::c_char;
use std::ffi::CString;

fn main() {
    let vars: HashMap<String, String> = std::env::vars().into_iter().collect();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "exit" {
            break;
        }
        if let Err(_) = run_process(&vars, input) {
            println!("Failed to execute command: {}", input);
        }
    }
}

struct Command<'c>(Vec<&'c str>);

impl<'c> Command<'c> {
    pub fn new(command: &'c str) -> Self {
        assert!(!command.is_empty(), "Commands cannot be empty!");
        Self(command.split_whitespace().collect())
    }

    pub fn bin_path(&self) -> &str {
        self.0[0] // Return the first element as the binary path
    }

    pub fn iter(&self) -> impl Iterator<Item = &'c str> {
        self.0.iter().copied()
    }
}

fn run_process(vars: &HashMap<String, String>, command: &str) -> Result<(), ()> {
    let command_parts = Command::new(command);
    run_shell_internals(&command_parts); // Call internal handling
    let bin = match find_binary(&command_parts, &vars["PATH"]) {
        Ok(b) => b,
        Err(err) => {
            println!("Failed to find {} : {}", command_parts.bin_path(), err);
            return Err(());
        }
    };

    match unsafe { libc::fork() } {
        -1 => panic!("Failed to start child process"),
        0 => {
            let pathname = CString::new(bin.to_str().expect("Only UTF-8 can be run")).unwrap();
            let argv_owned: Vec<CString> = command_parts.iter().map(|p| CString::new(*p).unwrap()).collect();
            let mut argv: Vec<*const c_char> = argv_owned.iter().map(|o| o.as_ptr()).collect();
            argv.push(std::ptr::null());
            let argv: *const *const c_char = argv.as_ptr();

            let envp_owned: Vec<CString> = vars
                .iter()
                .map(|(k, v)| {
                    let mut both = k.clone();
                    both.push_str("=");
                    both.push_str(v);
                    CString::new(both).expect("Null bytes not allowed in env string")
                })
                .collect();

            let mut envp: Vec<*const c_char> = envp_owned.iter().map(|o| o.as_ptr()).collect();
            envp.push(std::ptr::null());
            let envp: *const *const c_char = envp.as_ptr();

            unsafe {
                libc::execve(pathname.as_ptr(), argv, envp);
            }
            let err = std::io::Error::last_os_error();
            std::process::exit(0);
        }
        child_pid => {
            println!("Hello from parent. Child is {}", child_pid);
            let mut exit_code = 0;
            if unsafe { libc::waitpid(child_pid, &mut exit_code, 0) } == -1 {
                let err = std::io::Error::last_os_error();
                panic!("failed to wait: {:?}", err);
            }
            println!("Exited with {}", exit_code);
            return Ok(());
        }
    }
}

fn run_shell_internals(command: &Command) {
    match command.bin_path() {
        "cd" => {
            if command.0.len() > 1 {
                if let Err(e) = std::env::set_current_dir(command.0[1]) {
                    eprintln!("cd failed: {}", e);
                }
            }
        }
        "pwd" => {
            if let Ok(current_dir) = std::env::current_dir() {
                println!("{}", current_dir.display());
            }
        }
        "echo" => {
            let text = command.0.get(1).map_or("", |&s| s);
            println!("{}", text);
        }
        "clear" => {
            print!("\x1B[2J\x1B[1;1H");
        }
        "ls" => {
            let output = Command::new("ls").bin_path();
            println!("Executing ls: {}", output);
        }
        "mkdir" => {
            if let Some(dir) = command.0.get(1) {
                if let Err(e) = fs::create_dir(dir) {
                    eprintln!("mkdir failed: {}", e);
                }
            }
        }
        "rmdir" => {
            if let Some(dir) = command.0.get(1) {
                if let Err(e) = fs::remove_dir(dir) {
                    eprintln!("rmdir failed: {}", e);
                }
            }
        }
        "cp" => {
            if let (Some(src), Some(dest)) = (command.0.get(1), command.0.get(2)) {
                if let Err(e) = fs::copy(src, dest) {
                    eprintln!("cp failed: {}", e);
                }
            }
        }
        "mv" => {
            if let (Some(src), Some(dest)) = (command.0.get(1), command.0.get(2)) {
                if let Err(e) = fs::rename(src, dest) {
                    eprintln!("mv failed: {}", e);
                }
            }
        }
        "touch" => {
            if let Some(file) = command.0.get(1) {
                fs::File::create(file).unwrap_or_else(|_| {
                    eprintln!("Failed to create file: {}", file);
                    std::process::exit(1);
                });
            }
        }
        "cat" => {
            if let Some(file) = command.0.get(1) {
                match fs::read_to_string(file) {
                    Ok(contents) => println!("{}", contents),
                    Err(e) => eprintln!("cat failed: {}", e),
                }
            }
        }
        "nano" => {
            if let Some(file) = command.0.get(1) {
                // For simplicity, just print the file name, real implementation would need a call to an editor.
                println!("Opening nano: {}", file);
            }
        }
        "grep" => {
            if let (Some(pattern), Some(file_name)) = (command.0.get(1), command.0.get(2)) {
                let output = Command::new("grep").bin_path();
                println!("Executing grep: {} in file {}", pattern, file_name);
            }
        }
        "du" => {
            println!("Executing disk usage command...");
        }
        "df" => {
            println!("Executing disk free command...");
        }
        "ping" => {
            if let Some(host) = command.0.get(1) {
                println!("Pinging host: {}", host);
            }
        }
        "curl" => {
            if let Some(url) = command.0.get(1) {
                println!("Fetching URL: {}", url);
            }
        }
        _ => {
            eprintln!("Command not recognized: {}", command.bin_path());
        }
    }
}

fn find_binary(command: &Command, path: &str) -> Result<PathBuf, std::io::Error> {
    fn search(command: &str, path: &Path) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(path)? {
            if let Ok(entry) = entry {
                let met = entry.metadata()?;
                if met.is_file() || met.is_symlink() {
                    if let Some(name) = entry.path().file_name() {
                        if name == command {
                            if met.is_symlink() {
                                panic!("Running symlink not supported");
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(std::io::ErrorKind::NotFound.into())
    }

    // Check current directory first
    let target = command.bin_path();
    if let Ok(mut dir) = std::env::current_dir() {
        if let Ok(()) = search(target, &dir) {
            dir.push(target);
            return Ok(dir);
        }
    }

    // Search in PATH directories
    for entry in path.split(":") {
        let mut path = PathBuf::from(entry);
        if let Ok(()) = search(target, &path) {
            path.push(target);
            return Ok(path);
        }
    }

    Err(std::io::ErrorKind::NotFound.into())
}
