mod files;
mod tree;
use crate::{files::FileCollection, tree::RootFS};

use std::env::Args;

fn main() {
    let mut file_collection: FileCollection = parse_args(std::env::args());
    file_collection.squash();
    file_collection.expand();
    let root_fs = RootFS::from(&file_collection);
    println!("{root_fs}{file_collection}");
}

fn parse_args(args: Args) -> FileCollection {
    let mut file_collection: FileCollection = FileCollection::new();
    let iter: std::iter::Skip<Args> = args.skip(1);
    for arg in iter {
        if !arg.starts_with('-') {
            file_collection.add_rel(arg);
        } else if arg == "-s" {
            file_collection.secret = true;
        }
    }
    file_collection
}
