include!(concat!(env!("OUT_DIR"), "/commit_id.rs"));

fn main() {
    
    println!("--Sparrow mysql/mariadb proxy running!--");
    println!("commit_id: {}compile_time: {}", COMMIT_ID, COMPILE_TIME);
    println!("------------------------------------------------------");

}
