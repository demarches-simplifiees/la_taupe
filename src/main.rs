use std::env::args;

use la_taupe::{datamatrix::fetch_datamatrix, file_utils::file_to_img, twoddoc::parse};

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    let img = file_to_img(file_path);

    let datamatrix = fetch_datamatrix(img);

    if let Some(datamatrix) = datamatrix {
        let ddoc = parse(&datamatrix);
        println!("{:#?}", ddoc);
    }
}
