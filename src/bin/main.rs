#![expect(clippy::print_stdout, reason = "sanity check")]
use ::dimly::Schema as _;

fn main() {
    println!("StaticSchema: {}", ::dimly::StaticSchema::SCHEMA);
    println!("BorrowedSchema: {}", ::dimly::BorrowedSchema::SCHEMA);
    #[cfg(feature = "alloc")]
    println!("OwnedSchema: {}", ::dimly::OwnedSchema::SCHEMA);
}
