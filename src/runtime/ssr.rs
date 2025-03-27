use crate::components::{Component, StyledComponent};
use crate::styles::Styles;
use std::fmt::Write;

pub fn render_to_string(components: &[Box<dyn Component>]) -> String {
    let mut html = String::new();
    for comp in components {
        let styles = comp.styles();
        let mut style_str = String::new();
        for (k, v) in styles.props {
            write!(style_str, "{}: {};", k, v).unwrap();
        }
        match comp.as_any().type_id() {
            t if t == std::any::TypeId::of::<Text>() => {
                if let Some(text) = comp.as_any().downcast_ref::<Text>() {
                    writeln!(html, "<div style=\"{}\">{}</div>", style_str, text.content.get()).unwrap();
                }
            }
            t if t == std::any::TypeId::of::<Button>() => {
                if let Some(btn) = comp.as_any().downcast_ref::<Button>() {
                    writeln!(html, "<button style=\"{}\">{}</button>", style_str, btn.content.as_ref().map_or("", |c| c.get())).unwrap();
                }
            }
            // Add more built-in components as needed
            _ => {
                writeln!(html, "<div style=\"{}\">[Custom Component]</div>", style_str).unwrap();
            }
        }
    }
    html
}