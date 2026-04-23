use charts_rs::DEFAULT_FONT_DATA;
use resvg::{tiny_skia, usvg};
use std::sync::Arc;
use std::sync::LazyLock;
use thiserror::Error;
use usvg::fontdb;

#[derive(Debug, Error)]
pub enum SavingError {
    #[error("Image size is invalid, width: {width}, height: {height}")]
    Size { width: u32, height: u32 },
    #[error("Image encoding")]
    Image,
    #[error(transparent)]
    Parse(#[from] usvg::Error),
}

pub type SaveResult<T, E = SavingError> = std::result::Result<T, E>;

static GLOBAL_FONT_DB: LazyLock<Arc<fontdb::Database>> = LazyLock::new(|| {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_font_data(DEFAULT_FONT_DATA.to_vec());

    Arc::new(fontdb)
});

pub fn svg_to_png(svg: &str, w: u32) -> SaveResult<Vec<u8>> {
    let tree = usvg::Tree::from_str(
        svg,
        &usvg::Options { fontdb: GLOBAL_FONT_DB.clone(), ..Default::default() },
    )?;
    let pixmap_size = tree.size().to_int_size();

    let sx = w as f32 / pixmap_size.width() as f32;

    let sy = sx;
    let h = (pixmap_size.height() as f32 * sx).round() as u32;

    let mut pixmap =
        tiny_skia::Pixmap::new(w, h).ok_or_else(|| SavingError::Size {
            width: pixmap_size.width(),
            height: pixmap_size.height(),
        })?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );

    let data = pixmap.encode_png().map_err(|_| SavingError::Image)?;

    Ok(data)
}
