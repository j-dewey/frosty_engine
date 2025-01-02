mod snow;

fn run_name(name: &str) {
    match name {
        "snow" => snow::main(),
        _ => println!("Put in a valid example name"),
    }
}

fn main() {
    if let Some(name) = std::env::args().nth(1) {
        run_name(&name[..]);
    } else {
        println!("Put in an example name")
    }
}
