use std::fmt;
use std::process::exit;


pub struct ExitCode {
    value: i32,
    closed: bool,
}

impl ExitCode {
    pub fn new() -> ExitCode {
        ExitCode {
            value: 0,
            closed: false,
        }
    }
    pub fn report_error(&mut self) {
        self.value = 1;
    }
    pub fn is_ok(&self) -> bool {
        self.value == 0
    }
    pub fn error<D: fmt::Display>(&mut self, v: D) {
        error!("{}", v);
        self.value = 1;
    }
    pub fn fatal_context<A, B>(&mut self, a: A, v: B) -> !
        where A: fmt::Display, B: fmt::Display,
    {
        error!("{}: {}", a, v);
        self.closed = true;
        exit(1);
    }
    pub fn fatal_error<D: fmt::Display>(&mut self, v: D) -> ! {
        error!("{}", v);
        self.closed = true;
        exit(1);
    }
    pub fn exit(mut self) -> ! {
        self.closed = true;
        exit(self.value);
    }
    pub fn exit_if_failed(mut self) {
        self.closed = true;
        if self.value == 0 {
            return;
        }
        exit(self.value);
    }
}

impl Drop for ExitCode {
    fn drop(&mut self) {
        if !self.closed {
            panic!("Exit code is dropped");
        }
    }
}

