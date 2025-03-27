mod indexer; 

use std::fs::File;
use indexer::Indexer;

fn main() {
    let indexer = Indexer::new("hello".to_owned(), 123,1234, "abi".to_owned(), File::open("./abi.json").unwrap());
    println!("Hello {:?}", indexer);
}