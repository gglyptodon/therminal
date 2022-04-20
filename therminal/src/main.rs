fn main() {
    if let Err(e) = therminal::parse_args().and_then(therminal::run) {
        eprintln!("Error: {}", e);
    }
}
