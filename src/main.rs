use tpm_lib::{get_matches, handler};

fn main() {
    let args = std::env::args();
    let matches = get_matches(args);
    handler(&matches);
}
