use la_taupe::{datamatrix::fetch_datamatrix, file_utils::file_to_img, twoddoc::parse};

fn main() {
    let img = file_to_img("tests/fixtures/2ddoc/justificatif_de_domicile.pdf");

    let datamatrix = fetch_datamatrix(img);

    if let Some(datamatrix) = datamatrix {
        let ddoc = parse(&datamatrix);
        println!("{:#?}", ddoc);
    }
}
