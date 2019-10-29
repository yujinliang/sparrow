use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {

        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("commit_id.rs");
        let mut f = File::create(&dest_path).unwrap();

        //execute git.
        let commit = Command::new("git")
                                                         .arg("rev-parse")
                                                         .arg("HEAD")
                                                         .output()
                                                         .expect("Failed to execute git command");
        let commit = String::from_utf8(commit.stdout).expect("Invalid utf8 string");

        //create rust source code file.
        let  output = format!(r#"pub const COMMIT_ID : &'static str = "{}";"#, commit);
        f.write_all(output.as_bytes()).unwrap();

}