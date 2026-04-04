#![expect(clippy::print_stdout, reason = "sanity check")]
use schema::Schema as _;

fn main() {
    println!("StaticSchema: {}", ::schema::StaticSchema::SCHEMA);
    println!("BorrowedSchema: {}", ::schema::BorrowedSchema::SCHEMA);
    #[cfg(feature = "alloc")]
    println!("OwnedSchema: {}", ::schema::OwnedSchema::SCHEMA);
}
