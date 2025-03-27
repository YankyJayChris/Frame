use crate::runtime::Canvas;
use crate::styles::Styles;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub trait Component {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>);
    fn mount(&self) {}
    fn update(&self) {}
    fn unmount(&self) {}
    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None }
    fn on_click(&self) -> Option<&String> { None }
    fn on_touch_start(&self) -> Option<&String> { None }
    fn on_touch_move(&self) -> Option<&String> { None }
    fn on_touch_end(&self) -> Option<&String> { None }
    fn on_touch_cancel(&self) -> Option<&String> { None }
    fn styles(&self) -> Styles;
}

pub struct AppBar {
    pub title: Option<String>,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub menu: Option<(String, Box<dyn Fn()>)>,
    pub actions: Vec<Box<dyn Component>>,
    pub children: Vec<Rc<RefCell<Box<dyn Component>>>>, // New
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

impl Component for AppBar {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        let width = styles.get_px("width", 800);
        
        canvas.draw_rect(x, y, width, styles.get_px("height", 50), 0x333333);

        if let Some((icon, on_click)) = &self.menu {
            canvas.draw_svg(icon, x + 10, y + 15);
            on_click();
        }

        if let Some(title) = &self.title {
            canvas.draw_text(title, x + 50, y + 15, 0xFFFFFF);
        }

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

        let mut child_y = y + 50;
        for child in &self.children {
            let child_ref = child.borrow();
            let child_styles = child_ref.styles();
            let mut new_styles = child_styles.clone();
            new_styles.insert("x", &format!("{}dp", x + 5));
            new_styles.insert("y", &format!("{}dp", child_y));
            child_ref.render(canvas, &new_styles, animations);
            child_y += child_styles.get_px("height", 50) + 5;
        }
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { Some(&self.children) }
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct Text {
    pub content: String,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

impl Component for Text {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.content, x, y, 0xFFFFFF);
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // No children for Text
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct Image {
    pub src: String,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
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

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // No children for Image
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct Button {
    pub content: Option<String>,
    pub icon: Option<String>,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_click: Box<dyn Fn()>,                // Kept as closure
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
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

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // No children for Button
    fn on_click(&self) -> Option<&String> { None } // Closure-based, not string-based
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct List<'a> {
    pub data: &'a Vec<String>,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub build: Box<dyn Fn(&String) -> Box<dyn Component>>,
    pub children: Vec<Rc<RefCell<Box<dyn Component>>>>, // New
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

impl<'a> Component for List<'a> {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        let mut offset = 0;
        for item in self.data {
            let comp = (self.build)(item);
            comp.render(canvas, &Styles {
                props: {
                    let mut props = styles.props.clone();
                    props.insert("y".to_string(), (y + offset).to_string());
                    props
                }
            }, animations);
            offset += 20;
        }
        let mut child_y = y + offset;
        for child in &self.children {
            let child_ref = child.borrow();
            let child_styles = child_ref.styles();
            let mut new_styles = child_styles.clone();
            new_styles.insert("x", &format!("{}dp", x + 5));
            new_styles.insert("y", &format!("{}dp", child_y));
            child_ref.render(canvas, &new_styles, animations);
            child_y += child_styles.get_px("height", 50) + 5;
        }
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { Some(&self.children) }
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct BottomBar {
    pub current: i32,
    pub items: Vec<BottomBarItem>,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub children: Vec<Rc<RefCell<Box<dyn Component>>>>, // New
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

pub struct BottomBarItem {
    pub content: String,
    pub icon: String,
    pub on_click: Box<dyn Fn()>,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
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
        let mut child_y = y + 50;
        for child in &self.children {
            let child_ref = child.borrow();
            let child_styles = child_ref.styles();
            let mut new_styles = child_styles.clone();
            new_styles.insert("x", &format!("{}dp", x + 5));
            new_styles.insert("y", &format!("{}dp", child_y));
            child_ref.render(canvas, &new_styles, animations);
            child_y += child_styles.get_px("height", 50) + 5;
        }
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { Some(&self.children) }
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

impl Component for BottomBarItem {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.content, x, y, 0xFFFFFF);
        canvas.draw_svg(&self.icon, x, y - 20);
        (self.on_click)();
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // No children for BottomBarItem
    fn on_click(&self) -> Option<&String> { None }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct Input {
    pub value: String,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_change: Box<dyn Fn(&str)>,
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

impl Component for Input {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.value, x, y, 0xFFFFFF);
        (self.on_change)(&self.value);
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // No children for Input
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct Dropdown {
    pub options: Vec<String>,
    pub selected: usize,                        // Unique property
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_select: Box<dyn Fn(usize)>,          // Unique property
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

impl Component for Dropdown {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, _animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        canvas.draw_text(&self.options[self.selected], x, y, 0xFFFFFF);
        (self.on_select)(self.selected);
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // No children for Dropdown
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
}

pub struct Form {
    pub children: Vec<Box<dyn Component>>,
    pub styles: Styles,
    pub props: HashMap<String, String>,         // New
    pub on_submit: Box<dyn Fn()>,
    pub validation: String,
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,
    pub on_touch_move: Option<String>,
    pub on_touch_end: Option<String>,
    pub on_touch_cancel: Option<String>,
}

impl Component for Form {
    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<crate::runtime::Animation>) {
        let x = styles.get_px("x", 0);
        let y = styles.get_px("y", 0);
        let mut child_y = y;
        for child in &self.children {
            child.render(canvas, &Styles {
                props: {
                    let mut props = styles.props.clone();
                    props.insert("x".to_string(), (x + 5).to_string());
                    props.insert("y".to_string(), child_y.to_string());
                    props
                }
            }, animations);
            child_y += 50; // Adjust based on child height if needed
        }
        if self.validate() {
            (self.on_submit)();
        }
    }

    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> { None } // Children handled differently here
    fn on_click(&self) -> Option<&String> { self.on_click.as_ref() }
    fn on_touch_start(&self) -> Option<&String> { self.on_touch_start.as_ref() }
    fn on_touch_move(&self) -> Option<&String> { self.on_touch_move.as_ref() }
    fn on_touch_end(&self) -> Option<&String> { self.on_touch_end.as_ref() }
    fn on_touch_cancel(&self) -> Option<&String> { self.on_touch_cancel.as_ref() }
    fn styles(&self) -> Styles { self.styles.clone() }
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
impl StyledComponent for BottomBarItem { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Input { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Dropdown { fn styles(&self) -> Styles { self.styles.clone() } }
impl StyledComponent for Form { fn styles(&self) -> Styles { self.styles.clone() } }