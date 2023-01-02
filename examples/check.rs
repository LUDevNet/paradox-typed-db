use paradox_typed_db::TypedDatabase;
use std::fs;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);

    let file = args
        .next()
        .expect("USAGE: cargo run --example check -- cdclient.fdb");
    let bytes = fs::read(file)?;

    let db = assembly_fdb::mem::Database::new(&bytes);
    let tables = db.tables().expect("DB has tables");
    let _typed = TypedDatabase::new(tables).expect("Loading");

    let test = _typed.missions.row_iter().nth(100).unwrap();
    serde_json::to_writer_pretty(std::io::stdout(), &test)?;

    Ok(())
}
