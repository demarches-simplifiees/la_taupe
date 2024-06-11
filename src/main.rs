use la_taupe::{analysis::Analysis, http::server};
use serde_json::json;
use std::{env::args, path::Path};

fn main() {
    env_logger::init();

    let args: Vec<String> = args().collect();

    if args.contains(&String::from("--help")) {
        println!("La taupe: a tool to analyze files");
        println!("--version to print the version");
        println!("la_taupe file_path to analyze a file");
        println!("la_taupe to start the server");
        std::process::exit(0);
    }

    if args.contains(&String::from("--version")) {
        println!("Version: {}", env!("GIT_HASH"));
        std::process::exit(0);
    }

    if args.len() == 1 {
        let _ = server::main();
    } else {
        let path = Path::new(&args[1]);

        match Analysis::try_from(path) {
            Ok(analysis) => println!("{}", serde_json::to_string(&analysis).unwrap()),
            Err(msg) => {
                eprintln!("{}", json!({ "error": msg }));
                std::process::exit(1);
            }
        }
    }
}
