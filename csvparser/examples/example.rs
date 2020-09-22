use std::io::Read;
use csvparser::Csv;

fn main() -> anyhow::Result<()> {
    let mut buf = vec![];
    std::io::stdin().lock().read_to_end(&mut buf)?;
    let buf = String::from_utf8(buf)?;
    let mut csv = Csv::parse(&buf, true)?;
    for r in 0..csv.rows() {
        println!("{}", csv.vals(r).collect::<Vec<_>>().join(", "));
        csv.set_val(r, 0, "OK");
    }
    print!("{}", csv);
    Ok(())
}
