extern crate formatter;

use formatter::Formatter;

fn main() {
    let f = Formatter::from_root_config().expect("failed to initialize formatter");
    ::std::process::exit(match f.format_index() {
        Err(e) => {
            eprintln!("Failed to format index: {}", e);
            1
        }
        Ok(_) => 0,
    })
}
