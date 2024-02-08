use la_taupe::datamatrix::fetch_datamatrix;

fn main() {
    let result = fetch_datamatrix("tests/fixtures/2ddoc/justificatif_de_domicile.png");
    println!("{:?}", result);
}
