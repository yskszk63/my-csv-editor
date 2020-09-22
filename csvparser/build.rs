fn main() {
    println!("cargo:rerun-if-changed=src/csv.lalrpop");
    lalrpop::process_root().unwrap();
}
