include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));

fn main() {

    println!("commit_id: {}", COMMIT_ID);

}
