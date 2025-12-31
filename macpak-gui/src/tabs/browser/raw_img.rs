//! Custom RawImg View - Displays RGBA image data without PNG encoding

use floem::prelude::*;
use floem::peniko::{self, Blob};
use floem::{taffy, View, ViewId};
use floem_reactive::create_effect;
use floem_renderer::Renderer;
use std::sync::Arc;

/// Custom image view that works with raw RGBA pixel data
/// Uses a dynamic cache key so each image gets its own texture slot
pub struct RawImg {
    id: ViewId,
    img: Option<peniko::Image>,
    content_node: Option<taffy::tree::NodeId>,
    cache_key: u64,
}

/// Create a raw image view from RGBA data, width, height, and cache key
pub fn raw_img(width: u32, height: u32, rgba_data: Vec<u8>, cache_key: u64) -> RawImg {
    let data = Arc::new(rgba_data.into_boxed_slice());
    let blob = Blob::new(data);
    let image = peniko::Image::new(blob, peniko::Format::Rgba8, width, height);
    raw_img_dynamic(move || image.clone(), cache_key)
}

fn raw_img_dynamic(image: impl Fn() -> peniko::Image + 'static, cache_key: u64) -> RawImg {
    let id = ViewId::new();
    create_effect(move |_| {
        id.update_state(image());
    });
    RawImg {
        id,
        img: None,
        content_node: None,
        cache_key,
    }
}

impl View for RawImg {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "RawImg".into()
    }

    fn update(&mut self, _cx: &mut floem::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(img) = state.downcast::<peniko::Image>() {
            self.img = Some(*img);
            self.id.request_layout();
        }
    }

    fn layout(&mut self, cx: &mut floem::context::LayoutCx) -> taffy::tree::NodeId {
        cx.layout_node(self.id(), true, |_cx| {
            if self.content_node.is_none() {
                self.content_node = Some(self.id.new_taffy_node());
            }
            let content_node = self.content_node.unwrap();

            let (width, height) = self
                .img
                .as_ref()
                .map(|img| (img.width, img.height))
                .unwrap_or((0, 0));

            let style = floem::style::Style::new()
                .width((width as f64).px())
                .height((height as f64).px())
                .to_taffy_style();
            self.id.set_taffy_style(content_node, style);

            vec![content_node]
        })
    }

    fn paint(&mut self, cx: &mut floem::context::PaintCx) {
        if let Some(ref img) = self.img {
            let rect = self.id.get_content_rect();
            let hash_bytes = self.cache_key.to_le_bytes();
            cx.draw_img(
                floem_renderer::Img {
                    img: img.clone(),
                    hash: &hash_bytes,
                },
                rect,
            );
        }
    }
}