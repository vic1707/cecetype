#![expect(clippy::print_stdout, reason = "sanity check")]
use ::cecetype::Schema as _;

fn main() {
    println!("StaticSchema: {}", ::cecetype::StaticSchema::SCHEMA);
    println!("BorrowedSchema: {}", ::cecetype::BorrowedSchema::SCHEMA);
    #[cfg(feature = "alloc")]
    println!("OwnedSchema: {}", ::cecetype::OwnedSchema::SCHEMA);
}
