use super::{Element, LayoutMode, RenderContext};
use crate::layout::FixedBox;

/// Scaling / fitting mode for an image inside a `FixedImageBox`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ImageFit {
    /// Scale to fit entirely within the box, preserving aspect ratio.
    #[default]
    Contain,
    /// Scale to fill the box, preserving aspect ratio (may crop).
    Cover,
    /// Stretch to fill the box exactly (ignores aspect ratio).
    Stretch,
    /// Render at original pixel size; apply `OverflowPolicy` if it exceeds the box.
    Original,
}

/// An image element placed at a fixed position on the page.
///
/// Does not participate in `PageFlow`.
#[derive(Debug, Clone)]
pub struct FixedImageBox {
    pub image_box: FixedBox,
    /// Raw PNG or JPEG bytes.
    pub data: Vec<u8>,
    pub fit: ImageFit,
}

impl FixedImageBox {
    pub fn new(image_box: FixedBox, data: Vec<u8>) -> Self {
        Self {
            image_box,
            data,
            fit: ImageFit::Contain,
        }
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }
}

impl Element for FixedImageBox {
    fn layout_mode(&self) -> LayoutMode {
        LayoutMode::Fixed(self.image_box.clone())
    }

    fn estimated_height_mm(&self) -> f64 {
        0.0
    }

    fn render(&self, ctx: &mut RenderContext) -> crate::Result<super::RenderResult> {
        let ua = ctx.ua_config.enabled;
        if ua {
            match &self.image_box.ua_role {
                Some(tag) => {
                    let mcid = ctx.ua_tag_element(tag.clone(), self.image_box.ua_alt.clone());
                    ctx.backend.begin_tagged_content(tag.pdf_name().as_bytes(), mcid);
                }
                None => {
                    ctx.backend.begin_artifact_content();
                }
            }
        }

        if !self.data.is_empty() {
            if let Ok(img) = image::load_from_memory(&self.data) {
                let (px_w, px_h) = (img.width() as f64, img.height() as f64);
                let aspect = if px_w > 0.0 { px_h / px_w } else { 1.0 };
                let box_w = self.image_box.width_mm;
                let box_h = self.image_box.height_mm;

                let (render_w, render_h, x_off, y_off) = match self.fit {
                    ImageFit::Contain => {
                        let h_by_w = box_w * aspect;
                        if h_by_w <= box_h {
                            (box_w, h_by_w, 0.0, (box_h - h_by_w) / 2.0)
                        } else {
                            let rw = box_h / aspect;
                            (rw, box_h, (box_w - rw) / 2.0, 0.0)
                        }
                    }
                    ImageFit::Cover => {
                        let h_by_w = box_w * aspect;
                        if h_by_w >= box_h {
                            (box_w, h_by_w, 0.0, 0.0)
                        } else {
                            (box_h / aspect, box_h, 0.0, 0.0)
                        }
                    }
                    ImageFit::Stretch => (box_w, box_h, 0.0, 0.0),
                    ImageFit::Original => {
                        let w = (px_w * 25.4 / 96.0).min(box_w);
                        let h = (px_h * 25.4 / 96.0).min(box_h);
                        (w, h, 0.0, 0.0)
                    }
                };

                if render_w > 0.0 && render_h > 0.0 {
                    let img_ref = ctx.backend.embed_image(&self.data)?;
                    ctx.backend.draw_image(
                        img_ref,
                        self.image_box.x_mm + x_off,
                        self.image_box.y_mm + y_off,
                        render_w,
                        render_h,
                    );
                }
            }
        }

        if ua { ctx.backend.end_tagged_content(); }
        Ok(super::RenderResult::done())
    }
}
