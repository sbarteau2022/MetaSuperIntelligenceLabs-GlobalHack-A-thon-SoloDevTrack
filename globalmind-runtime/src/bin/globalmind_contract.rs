use globalmind_runtime::{GlobalMindStore, StoreConfig};

fn run() -> Result<(), String> {
    let store = GlobalMindStore::connect(StoreConfig::from_env())?;
    store.verify_contract()?;
    println!("GlobalMind SurrealDB contract verified");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("GlobalMind contract verification failed: {error}");
        std::process::exit(1);
    }
}
