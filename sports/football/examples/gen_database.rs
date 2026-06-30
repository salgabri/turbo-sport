//! Generate the sample starting database to a JSON file — the file the editor app opens and
//! the game starts from.
//!
//! Run: `cargo run -p football --example gen_database [path]`  (default: database.json)

fn main() {
    let db = football::sample();
    db.validate().expect("sample database is valid");

    let path = std::env::args().nth(1).unwrap_or_else(|| "database.json".to_string());
    football::database::save(&db, &path).expect("failed to write database");

    println!(
        "wrote '{}' to {path}: {} divisions, {} clubs, {} players",
        db.name,
        db.divisions.len(),
        db.clubs.len(),
        db.players.len(),
    );
}
