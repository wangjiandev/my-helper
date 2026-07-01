//! 合成最终 PDF：火车票 4×2 排版、普通发票 2×1 排版、每页中心裁切虚线。

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use pdfium_render::prelude::*;

use crate::invoice::Invoice;

/// A4 竖版尺寸（pt）。
fn a4() -> PdfPagePaperSize {
    PdfPagePaperSize::a4()
}

/// 合成 PDF 并保存到 `out`，返回总页数。
pub fn compose(pdfium: &Pdfium, invoices: &[Invoice], out: &Path) -> Result<usize> {
    let trains: Vec<&Invoice> = invoices.iter().filter(|i| i.kind.is_train()).collect();
    let regulars: Vec<&Invoice> = invoices.iter().filter(|i| !i.kind.is_train()).collect();

    let mut doc = pdfium.create_new_pdf()?;

    add_tiled(pdfium, &mut doc, &trains, 4, 2, "火车票")?;
    add_tiled(pdfium, &mut doc, &regulars, 2, 1, "普通发票")?;

    // 每页绘制水平中心虚线
    let total = doc.pages().len();
    for idx in 0..total {
        let mut page = doc
            .pages()
            .get(idx)
            .map_err(|e| anyhow!("获取页面失败: {e}"))?;
        draw_center_line(&mut page)?;
    }

    doc.save_to_file(out)
        .map_err(|e| anyhow!("保存 PDF 失败: {e}"))?;

    Ok(total as usize)
}

/// 把若干发票页排版成 `rows × cols` 网格的 A4 页，并追加到目标文档。
fn add_tiled(
    pdfium: &Pdfium,
    dest: &mut PdfDocument,
    invoices: &[&Invoice],
    rows: u8,
    cols: u8,
    label: &str,
) -> Result<()> {
    if invoices.is_empty() {
        return Ok(());
    }
    let src = build_source(pdfium, invoices)?;
    let pages = src.pages();
    let tiled = pages
        .tile_into_new_document(rows, cols, a4())
        .map_err(|e| anyhow!("{label}排版失败: {e}"))?;
    dest.pages_mut()
        .append(&tiled)
        .map_err(|e| anyhow!("合并{label}页失败: {e}"))?;
    Ok(())
}

/// 构建一个仅含各发票首页的源文档（供 tile 使用）。
fn build_source<'a>(pdfium: &'a Pdfium, invoices: &'a [&'a Invoice]) -> Result<PdfDocument<'a>> {
    let mut src = pdfium.create_new_pdf()?;
    for inv in invoices {
        let single = pdfium
            .load_pdf_from_file(&inv.path, None)
            .with_context(|| format!("打开 PDF 失败: {}", inv.path.display()))?;
        src.pages_mut()
            .append(&single)
            .map_err(|e| anyhow!("拼接页面失败: {e}"))?;
    }
    Ok(src)
}

/// 在页面水平中心线（H/2）绘制贯穿左右的虚线，便于裁切成两片 A5。
fn draw_center_line(page: &mut PdfPage<'_>) -> Result<()> {
    let w = page.width().value;
    let h = page.height().value;
    let y = h / 2.0;
    let color = PdfColor::new(130, 130, 130, 255);
    let (dash, gap) = (4.0_f32, 4.0_f32);
    let mut x = 0.0_f32;
    while x < w {
        let x2 = (x + dash).min(w);
        page.objects_mut()
            .create_path_object_line(
                PdfPoints::new(x),
                PdfPoints::new(y),
                PdfPoints::new(x2),
                PdfPoints::new(y),
                color,
                PdfPoints::new(0.6),
            )
            .map_err(|e| anyhow!("绘制中心线失败: {e}"))?;
        x += dash + gap;
    }
    Ok(())
}
