use crate::runtime::Canvas;
use crate::styles::Styles;

pub trait Component {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>);
}

pub struct AppBar {
    pub title: Option<String>,
    pub styles: Styles,
    pub menu: Option<(String, Box<dyn Fn()>)>, // (icon, on_click)
    pub actions: Vec<Box<dyn Component>>,
}

impl Component for AppBar {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        let width = styles.get_px("width", 800);
        
        // Draw background
        canvas.draw_rect(x, y, width, styles.get_px("height", 50), 0x333333);

        // Draw menu icon if present
        if let Some((icon, on_click)) = &self.menu {
            canvas.draw_svg(icon, x + 10, y + 15);
            on_click();
        }

        // Draw title if present
        if let Some(title) = &self.title {
            canvas.draw_text(title, x + 50, y + 15, 0xFFFFFF);
        }

        // Draw actions
        let mut action_x = width - (self.actions.len() as i32 * 40);
        for action in &self.actions {
            action.render(canvas, &Styles {
                props: {
                    let mut props = styles.props.clone();
                    props.insert("x".to_string(), (x + action_x).to_string());
                    props.insert("y".to_string(), (y + 15).to_string());
                    props
                }
            }, animations);
            action_x += 40;
        }
    }
}

pub struct Text {
    pub content: String,
    pub styles: Styles,
}

impl Component for Text {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.content, x, y, 0xFFFFFF);
    }
}

pub struct Image {
    pub src: String,
    pub styles: Styles,
}

impl Component for Image {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        if self.src.ends_with(".svg") {
            canvas.draw_svg(&self.src, x, y);
        } else {
            canvas.draw_image(&self.src, x, y);
        }
    }
}

pub struct Button {
    pub content: Option<String>,
    pub icon: Option<String>,
    pub styles: Styles,
    pub on_click: Box<dyn Fn()>,
}

impl Component for Button {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        if let Some(content) = &self.content {
            canvas.draw_text(content, x, y, 0x00FF00);
        }
        if let Some(icon) = &self.icon {
            canvas.draw_svg(icon, x, y);
        }
        (self.on_click)();
    }
}

pub struct List<'a> {
    pub data: &'a Vec<String>,
    pub styles: Styles,
    pub build: Box<dyn Fn(&String) -> Box<dyn Component>>,
}

impl<'a> Component for List<'a> {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        let mut offset = 0;
        for item in self.data {
            let comp = (self.build)(item);
            comp.render(canvas, styles, animations);
            offset += 20; // Flexbox replaces this
        }
    }
}

pub struct BottomBar {
    pub current: i32,
    pub items: Vec<BottomBarItem>,
    pub styles: Styles,
}

pub struct BottomBarItem {
    pub content: String,
    pub icon: String,
    pub on_click: Box<dyn Fn()>,
    pub styles: Styles,
}

impl Component for BottomBar {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 600 - 50);
        let mut offset = 0;
        for (i, item) in self.items.iter().enumerate() {
            let color = if i as i32 == self.current { 0xFFFF00 } else { 0xFFFFFF };
            canvas.draw_text(&item.content, x + offset, y, color);
            canvas.draw_svg(&item.icon, x + offset, y - 20);
            (item.on_click)();
            offset += 100;
        }
    }
}

pub struct Input {
    pub value: String,
    pub styles: Styles,
    pub on_change: Box<dyn Fn(&str)>,
}

impl Component for Input {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.value, x, y, 0xFFFFFF);
        // Simulate input change (in a real app, this would tie to winit events)
        (self.on_change)(&self.value);
    }
}

pub struct Dropdown {
    pub options: Vec<String>,
    pub selected: usize,
    pub styles: Styles,
    pub on_select: Box<dyn Fn(usize)>,
}

impl Component for Dropdown {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.options[self.selected], x, y, 0xFFFFFF);
        // Simulate selection (in a real app, this would tie to winit events)
        (self.on_select)(self.selected);
    }
}

pub struct Form {
    pub children: Vec<Box<dyn Component>>,
    pub styles: Styles,
    pub on_submit: Box<dyn Fn()>,
    pub validation: String,
}

impl Component for Form {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        for child in &self.children {
            child.render(canvas, styles, animations);
        }
        if self.validate() {
            (self.on_submit)();
        }
    }
}

impl Form {
    fn validate(&self) -> bool {
        self.validation == "required" && !self.children.iter().any(|c| c.as_ref().to_string().is_empty())
    }
}

pub trait StyledComponent {
    fn styles(&self) -> Styles;
}

impl<T: Component> StyledComponent for T {
    fn styles(&self) -> Styles { Styles::default() }
}

impl StyledComponent for AppBar { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Text { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Image { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Button { fn styles(&self) -> Styles { self.styles.clone() } }
impl<'a> StyledComponent for List<'a> { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for BottomBar { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Input { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Dropdown { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Form { fn styles(&self) -> Styles { self.styles.clone() } }