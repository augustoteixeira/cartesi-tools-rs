use echo_lib;
use trolley::cmt;

fn main() {
    println!("Hello, World!");
    let rollup = cmt::RollupCmt::new();
    echo_lib::run(rollup);
}
