#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_extra)]

// mod query;
// use query::lexer::Lexer;
mod bplus_tree;
use bplus_tree::BPlusTree;
mod storage;

use std::collections::BTreeMap;

use rand::prelude::*;

fn main() {
    // let mut ret = Lexer::new("select * from hoge;").exec();
    // println!("{:?}", ret);
    // ret = Lexer::new("select * from hoge where age>=10;").exec();
    // println!("{:?}", ret);
    // ret = Lexer::new("INSERT INTO hoge VALUES (\"name\", 20);").exec();
    // println!("{:?}", ret);

    // let mut btree = BTreeMap::<usize, &str>::new();
    let mut btree = BPlusTree::<usize, &str>::new();

    // btree.insert(9usize, "data");
    // // let ret = btree.get(&9);
    // // println!("{:?}",ret);
    // btree.insert(20, "data");
    // btree.insert(10, "data");
    // btree.insert(15, "data");
    // btree.insert(18, "data");
    // btree.insert(5, "data");

    // btree.insert(1, "data");
    // btree.insert(11, "data");
    // btree.insert(50, "data");
    
    // btree.insert(22, "data");
    // btree.insert(13, "data");


    let mut rng = rand::thread_rng();
    let mut insert_dataset = vec![];
    for _ in 0..18 {
        let key = rng.gen::<u8>();
        insert_dataset.push(key as usize);
    }
    println!("{:?}", insert_dataset);
    for key in insert_dataset {
        btree.insert(key, "data");
    }
    println!("{:?}", btree);
}
