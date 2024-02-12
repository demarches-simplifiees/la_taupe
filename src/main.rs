use la_taupe::{datamatrix::fetch_datamatrix, twoddoc::parse};

fn main() {
    let datamatrix = fetch_datamatrix("tests/fixtures/2ddoc/justificatif_de_domicile.png");

    if let Some(datamatrix) = datamatrix {
        let ddoc = parse(&datamatrix);
        println!("{:#?}", ddoc);
    }
}
