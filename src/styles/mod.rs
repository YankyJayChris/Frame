use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Styles {
    pub props: HashMap<String, String>,
}

impl Styles {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }

    pub fn get_px(&self, key: &str, default: i32) -> i32 {
        self.props.get(key)
            .and_then(|v| v.trim_end_matches(|c| c == '%' || c == 'd' || c == 'p').parse::<i32>().ok())
            .unwrap_or(default)
    }

    pub fn layout(components: &[Box<dyn crate::components::Component>]) -> Layout {
        let mut layout = Layout {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            children: Vec::new(),
            flex_direction: "row".to_string(),
            grid_template: None,
        };
        for comp in components {
            layout.children.push(comp.clone());
            if let Some(direction) = comp.as_ref().styles().get("direction") {
                layout.flex_direction = direction.clone();
            }
            if let Some(template) = comp.as_ref().styles().get("grid-template") {
                layout.grid_template = Some(template.clone());
            }
        }
        layout
    }

    pub fn with_defaults() -> Self {
        let mut props = HashMap::new();
        props.insert("padding".to_string(), "10px".to_string());
        props.insert("margin".to_string(), "5px".to_string());
        props.insert("font-size".to_string(), "16px".to_string());
        props.insert("color".to_string(), "#FFFFFF".to_string());
        Styles { props }
    }
}

pub struct Layout {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    children: Vec<Box<dyn crate::components::Component>>,
    flex_direction: String,
    grid_template: Option<String>,
}

impl Layout {
    pub fn render(&self, canvas: &mut Canvas, animations: &mut Vec<crate::runtime::Animation>) {
        if let Some(template) = &self.grid_template {
            self.render_grid(canvas, animations, template);
        } else {
            self.render_flex(canvas, animations);
        }
    }

    fn render_flex(&self, canvas: &mut Canvas, animations: &mut Vec<crate::runtime::Animation>) {
        let mut offset = 0;
        for child in &self.children {
            let styles = child.as_ref().styles();
            let x = if self.flex_direction == "row" { self.x + offset } else { self.x };
            let y = if self.flex_direction == "column" { self.y + offset } else { self.y };
            let width = styles.get_px("width", 100);
            let height = styles.get_px("height", 50);
            child.render(canvas, &Styles {
                props: {
                    let mut props = styles.props.clone();
                    props.insert("x".to_string(), x.to_string());
                    props.insert("y".to_string(), y.to_string());
                    props
                }
            }, animations);
            offset += if self.flex_direction == "row" { width } else { height };
        }
    }

    fn render_grid(&self, canvas: &mut Canvas, animations: &mut Vec<crate::runtime::Animation>, template: &str) {
        let rows: Vec<&str> = template.split('/').collect();
        let row_height = self.height / rows.len() as i32;
        let mut y_offset = 0;
        for (i, row) in rows.iter().enumerate() {
            let cols: Vec<&str> = row.split(' ').collect();
            let col_width = self.width / cols.len() as i32;
            let mut x_offset = 0;
            for (j, _col) in cols.iter().enumerate() {
                if let Some(child) = self.children.get(i * cols.len() + j) {
                    let styles = child.as_ref().styles();
                    child.render(canvas, &Styles {
                        props: {
                            let mut props = styles.props.clone();
                            props.insert("x".to_string(), (self.x + x_offset).to_string());
                            props.insert("y".to_string(), (self.y + y_offset).to_string());
                            props
                        }
                    }, animations);
                }
                x_offset += col_width;
            }
            y_offset += row_height;
        }
    }

    pub fn styles(&self) -> Styles {
        Styles::with_defaults()
    }
}