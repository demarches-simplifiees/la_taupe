use std::env::args;

use la_taupe::{
    datamatrix::fetch_datamatrix, file_utils::file_to_img, http::server, twoddoc::parse,
};

fn main() {
    env_logger::init();

    let args: Vec<String> = args().collect();

    if args.len() == 1 {
        let _ = server::main();
    } else {
        let file_path = &args[1];

        let img = file_to_img(file_path);

        let datamatrix = fetch_datamatrix(img);

        if let Some(datamatrix) = datamatrix {
            let ddoc = parse(&datamatrix);
            println!("{:#?}", ddoc);
        }
    }
}
