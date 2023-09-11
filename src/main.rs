use mocasa::run;

fn main() {
    match run() {
        Ok(_) => { println!("Done!") }
        Err(error) => { println!("Error: {}", error)}
    }
}
