# chibiKVS
a toy database management system written in Rust-lang.  
this's not work yet.

2021/04/18:   
this imitate mini-redis-server that well known as tokio tutorial.
https://tokio.rs/tokio/tutorial

# Usage
1. Download & Execute 
```bash
$ git clone https://github.com/totechite/chibikvs.git
$ cd ./chibikvs
$ cargo run 
Listening
```
2. Install mini redis Client 
```bash
cargo install mini-redis
```
3. You can try some operator.
```
$ mini-redis-cli set foo bar
OK
$ mini-redis-cli get foo
"bar"
```

### LICENSE
MIT