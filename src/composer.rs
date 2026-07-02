//! 合成最终 PDF：火车票 4×2 排版（相邻票间留空白间隔）、普通发票 2×1 排版、每页中心裁切虚线。

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use pdfium_render::prelude::*;

use crate::invoice::Invoice;

/// 火车票相邻票之间的空白间隔（pt）。
const TRAIN_GAP: f32 = 12.0;
/// 每页火车票行数。
const TRAIN_ROWS: usize = 4;
/// 每页火车票列数。
const TRAIN_COLS: usize = 2;

/// A4 竖版尺寸（pt）。
fn a4() -> PdfPagePaperSize {
    PdfPagePaperSize::a4()
}

/// 合成 PDF 并保存到 `out`，返回总页数。
pub fn compose(pdfium: &Pdfium, invoices: &[Invoice], out: &Path) -> Result<usize> {
    let trains: Vec<&Invoice> = invoices.iter().filter(|i| i.kind.is_train()).collect();
    let regulars: Vec<&Invoice> = invoices.iter().filter(|i| !i.kind.is_train()).collect();

    let mut doc = pdfium.create_new_pdf()?;

    add_trains_gapped(pdfium, &mut doc, &trains)?;
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

/// 火车票按 4×2 排版，相邻票之间留 `TRAIN_GAP` 空白间隔，外边缘贴页面边沿。
/// 使用 XObject Form 放置各票（保留矢量质量），等比缩放并居中到每个单元格。
fn add_trains_gapped<'a>(
    pdfium: &'a Pdfium,
    dest: &mut PdfDocument<'a>,
    trains: &[&Invoice],
) -> Result<()> {
    if trains.is_empty() {
        return Ok(());
    }

    let page_w = a4().width().value;
    let page_h = a4().height().value;
    let g = TRAIN_GAP;

    // 列：两列之间留一个间隔，左右贴边。
    let col_w = (page_w - g) / 2.0;
    // 行：保持与水平中心线（H/2）对称——上、下半 A5 各容纳 2 行，半内两行间留一个间隔。
    let half_h = page_h / 2.0;
    let row_h = (half_h - g) / 2.0;

    let per_page = TRAIN_ROWS * TRAIN_COLS;
    for chunk in trains.chunks(per_page) {
        let mut page = dest
            .pages_mut()
            .create_page_at_end(a4())
            .map_err(|e| anyhow!("创建火车票页失败: {e}"))?;

        for (i, inv) in chunk.iter().enumerate() {
            let row = i / TRAIN_COLS;
            let col = i % TRAIN_COLS;

            // 单元格左下角坐标（PDF 坐标，原点左下、Y 向上）。
            let cell_x = if col == 0 { 0.0 } else { col_w + g };
            let cell_y = row_bottom(row, page_h, row_h, g);

            let single = pdfium
                .load_pdf_from_file(&inv.path, None)
                .with_context(|| format!("打开 PDF 失败: {}", inv.path.display()))?;
            let src_page = single
                .pages()
                .get(0)
                .map_err(|e| anyhow!("读取发票页面失败: {e}"))?;
            let src_w = src_page.width().value.max(1.0);
            let src_h = src_page.height().value.max(1.0);

            // 等比缩放以放入单元格，并居中。
            let scale = (col_w / src_w).min(row_h / src_h);
            let w_s = src_w * scale;
            let h_s = src_h * scale;
            let tx = cell_x + (col_w - w_s) / 2.0;
            let ty = cell_y + (row_h - h_s) / 2.0;

            let mut form = src_page
                .objects()
                .copy_into_x_object_form_object(dest)
                .map_err(|e| anyhow!("转换发票页面失败: {e}"))?;
            form.transform(scale, 0.0, 0.0, scale, tx, ty)
                .map_err(|e| anyhow!("定位发票失败: {e}"))?;
            page.objects_mut()
                .add_object(form)
                .map_err(|e| anyhow!("写入发票失败: {e}"))?;
        }
    }

    Ok(())
}

/// 给定行号（0=最上一行），返回该行单元格的底部 y 坐标。
fn row_bottom(row: usize, page_h: f32, row_h: f32, g: f32) -> f32 {
    match row {
        0 => page_h - row_h,
        1 => page_h - 2.0 * row_h - g,
        2 => row_h + g,
        _ => 0.0, // row 3 贴底部
    }
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
