use std::process::exit;


pub struct ExitCode {
    value: i32,
}

impl ExitCode {
    pub fn new() -> ExitCode {
        ExitCode {
            value: 0,
        }
    }
    pub fn report_error(&mut self) {
        self.value = 1;
    }
    pub fn exit(self) -> ! {
        exit(self.value);
    }
}

impl Drop for ExitCode {
    fn drop(&mut self) {
        panic!("Exit code is dropped");
    }
}

