use pdfium_render::prelude::*;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Path::new("./lib/libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    for f in ["f00017", "f00020"] {
        let path = format!("./test/{f}.pdf");
        let doc = pdfium.load_pdf_from_file(&path, None)?;
        let page = doc.pages().get(0)?;
        let txt = page.text()?.all();
        println!("========== {f} (len={}) ==========", txt.len());
        // show repr to reveal whitespace/newlines
        for line in txt.split('\n') {
            println!("|{line}|");
        }
    }
    Ok(())
}
