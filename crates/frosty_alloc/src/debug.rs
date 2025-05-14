use std::fs::File;

// A simple trait describing how an object dumps
// debug data to a file
pub trait DebugOutter {
    fn dump_data(&self, fs: &mut File);
}

// A simple trait descirbing how to get debug
// data from an object
pub trait DebugData {
    fn get_debug(&self) -> String;
}
