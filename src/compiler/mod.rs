use crate::parser::{AST, Component as ParserComponent, Expr, Animation as ParserAnimation};
use std::fmt::Write;

pub fn compile(ast: AST) -> String {
    let mut code = String::new();

    writeln!(code, "use frame::runtime::{{run_app, CanvasApp, State, Navigation, Animation, Reactive, render_to_string, I18n, Canvas, Styles}};").unwrap();
    writeln!(code, "use std::collections::HashMap;").unwrap();
    writeln!(code, "use tokio::runtime::Runtime;").unwrap();
    writeln!(code, "use std::rc::Rc;").unwrap();
    writeln!(code, "use std::cell::RefCell;").unwrap();
    writeln!(code, "use std::panic;").unwrap();
    writeln!(code, "#[cfg(not(debug_assertions))]").unwrap();
    writeln!(code, "use std::mem;").unwrap();

    writeln!(code, "lazy_static::lazy_static! {{ static ref VARS: HashMap<String, String> = {:#?}; }}", ast.vars).unwrap();
    writeln!(code, "lazy_static::lazy_static! {{ static ref I18N: HashMap<String, String> = {:#?}; }}", ast.i18n).unwrap();

    for (name, comp_def) in &ast.components {
        writeln!(code, "#[derive(Clone)]").unwrap();
        writeln!(code, "struct {} {{", name).unwrap();
        writeln!(code, "    name: String,").unwrap();
        writeln!(code, "    styles: Styles,").unwrap();
        writeln!(code, "    dirty: bool,").unwrap();
        if comp_def.content.is_some() { writeln!(code, "    content: Reactive<String>,").unwrap(); }
        if !comp_def.children.is_empty() { writeln!(code, "    children: Vec<Rc<RefCell<Box<dyn Component>>>>,").unwrap(); }
        if comp_def.on_click.is_some() { writeln!(code, "    on_click: Option<String>,").unwrap(); }
        if comp_def.on_mount.is_some() { writeln!(code, "    on_mount: Option<String>,").unwrap(); }
        if comp_def.on_update.is_some() { writeln!(code, "    on_update: Option<String>,").unwrap(); }
        if comp_def.on_unmount.is_some() { writeln!(code, "    on_unmount: Option<String>,").unwrap(); }
        if comp_def.data.is_some() { writeln!(code, "    data: Reactive<String>,").unwrap(); }
        if comp_def.build.is_some() { writeln!(code, "    build: Box<dyn Fn(&str) -> Box<dyn Component>>,").unwrap(); }
        for (prop_name, (prop_type, _)) in &comp_def.props {
            writeln!(code, "    {}: {},", prop_name, match prop_type.as_str() {
                "string" => "String",
                "number" => "f64",
                "bool" => "bool",
                _ => "String",
            }).unwrap();
        }
        writeln!(code, "}}").unwrap();

        writeln!(code, "impl {} {{", name).unwrap();
        writeln!(code, "    fn new(props: HashMap<String, String>) -> Self {{", name).unwrap();
        writeln!(code, "        let mut comp = Self {{", name).unwrap();
        writeln!(code, "            name: \"{}\".to_string(),", name).unwrap();
        writeln!(code, "            styles: Styles::new(),").unwrap();
        writeln!(code, "            dirty: true,").unwrap();
        if comp_def.content.is_some() { writeln!(code, "            content: Reactive::new(String::new()),").unwrap(); }
        if !comp_def.children.is_empty() { writeln!(code, "            children: Vec::new(),").unwrap(); }
        if comp_def.on_click.is_some() { writeln!(code, "            on_click: None,").unwrap(); }
        if comp_def.on_mount.is_some() { writeln!(code, "            on_mount: None,").unwrap(); }
        if comp_def.on_update.is_some() { writeln!(code, "            on_update: None,").unwrap(); }
        if comp_def.on_unmount.is_some() { writeln!(code, "            on_unmount: None,").unwrap(); }
        if comp_def.data.is_some() { writeln!(code, "            data: Reactive::new(String::new()),").unwrap(); }
        if comp_def.build.is_some() {
            let (param, build_comp) = comp_def.build.as_ref().unwrap();
            writeln!(code, "            build: Box::new(|{}| Box::new({})),", param, compile_component(build_comp, 0, 0, &ast)).unwrap();
        }
        for (prop_name, (prop_type, required)) in &comp_def.props {
            let rust_type = match prop_type.as_str() {
                "string" => "String",
                "number" => "f64",
                "bool" => "bool",
                _ => "String",
            };
            if *required {
                writeln!(code, "            {}: props.get(\"{}\").expect(\"Missing required prop {}\").parse::<{}>().unwrap(),", prop_name, prop_name, prop_name, rust_type).unwrap();
            } else {
                writeln!(code, "            {}: props.get(\"{}\").map_or({}::default(), |v| v.parse::<{}>().unwrap_or_default()),", prop_name, prop_name, rust_type, rust_type).unwrap();
            }
        }
        writeln!(code, "        }};").unwrap();
        if let Some(content) = &comp_def.content {
            writeln!(code, "        comp.content.set({});", if content.starts_with("t:") { format!("I18N.get(\"{}\").unwrap_or(&String::new()).clone()", content.trim_start_matches("t:").trim_matches('"')) } else { format!("\"{}\".to_string()", content.trim_matches('"')) }).unwrap();
        }
        if let Some(click) = &comp_def.on_click { writeln!(code, "        comp.on_click = Some(\"{}\".to_string());", click); }
        if let Some(mount) = &comp_def.on_mount { writeln!(code, "        comp.on_mount = Some(\"{}\".to_string());", mount); }
        if let Some(update) = &comp_def.on_update { writeln!(code, "        comp.on_update = Some(\"{}\".to_string());", update); }
        if let Some(unmount) = &comp_def.on_unmount { writeln!(code, "        comp.on_unmount = Some(\"{}\".to_string());", unmount); }
        writeln!(code, "        comp.styles = Styles {{ props: {:#?} }};", resolve_styles(&comp_def.styles, &ast.vars)).unwrap();
        writeln!(code, "        comp").unwrap();
        writeln!(code, "    }}").unwrap();
        writeln!(code, "}}").unwrap();

        writeln!(code, "impl Component for {} {{", name).unwrap();
        writeln!(code, "    fn render(&self, canvas: &mut Canvas, styles: &Styles, animations: &mut Vec<Animation>) {{", name).unwrap();
        writeln!(code, "        if !self.dirty {{ return; }}").unwrap();
        writeln!(code, "        let x = styles.get_px(\"x\", 0);").unwrap();
        writeln!(code, "        let y = styles.get_px(\"y\", 0);").unwrap();
        writeln!(code, "        let w = styles.get_px(\"width\", {});", if name == "AppBar" || name == "BottomBar" { "canvas.width as i32" } else { "100" }).unwrap();
        writeln!(code, "        let h = styles.get_px(\"height\", 50);").unwrap();
        writeln!(code, "        let bg = styles.get_hex(\"background\", 0xFFFFFFFF);").unwrap();
        writeln!(code, "        let color = styles.get_hex(\"color\", 0xFFFFFF);").unwrap();
        writeln!(code, "        canvas.draw_rect(x, y, w, h, bg);").unwrap();
        writeln!(code, "        match self.name.as_str() {{", name).unwrap();
        writeln!(code, "            \"Text\" => canvas.draw_text(&self.content.get(), x + 5, y + 15, color),").unwrap();
        writeln!(code, "            \"Button\" => canvas.draw_text(&self.content.get(), x + 5, y + 15, 0xFFFFFF),").unwrap();
        writeln!(code, "            \"input\" => canvas.draw_text(&self.content.get(), x + 5, y + 15, 0xFF000000),").unwrap();
        writeln!(code, "            \"list\" => {{ /* Render children dynamically later */ }}").unwrap();
        writeln!(code, "            _ => println!(\"Rendering custom component: {{}}\", self.name),").unwrap();
        writeln!(code, "        }}").unwrap();
        if !comp_def.children.is_empty() {
            writeln!(code, "            let mut child_y = y;").unwrap();
            writeln!(code, "            for child in &self.children {{", name).unwrap();
            writeln!(code, "                let child_ref = child.borrow();").unwrap();
            writeln!(code, "                let child_styles = child_ref.styles();").unwrap();
            writeln!(code, "                let mut new_styles = child_styles.clone();").unwrap();
            writeln!(code, "                new_styles.insert(\"x\", &format!(\"{{}}dp\", x + 5));").unwrap();
            writeln!(code, "                new_styles.insert(\"y\", &format!(\"{{}}dp\", child_y));").unwrap();
            writeln!(code, "                child_ref.render(canvas, &new_styles, animations);").unwrap();
            writeln!(code, "                child_y += child_styles.get_px(\"height\", 50) + 5;").unwrap();
            writeln!(code, "            }}").unwrap();
        }
        if let Some(anim) = &comp_def.animate {
            writeln!(code, "            animations.push(Animation::new(\"{}\", {}, Styles {{ props: {:#?} }}, Styles {{ props: {:#?} }}));", 
                anim.kind, anim.duration, anim.from.props, anim.to.props).unwrap();
        }
        writeln!(code, "    }}").unwrap();
        writeln!(code, "    fn mount(&self) {{ if let Some(m) = &self.on_mount {{ State::new().call(m); }} }}").unwrap();
        writeln!(code, "    fn update(&self) {{ if let Some(u) = &self.on_update {{ State::new().call(u); }} }}").unwrap();
        writeln!(code, "    fn unmount(&self) {{ if let Some(u) = &self.on_unmount {{ State::new().call(u); }} }}").unwrap();
        writeln!(code, "    fn children(&self) -> Option<&Vec<Rc<RefCell<Box<dyn Component>>>>> {{ Some(&self.children) }}").unwrap();
        writeln!(code, "    fn on_click(&self) -> Option<&String> {{ self.on_click.as_ref() }}").unwrap();
        writeln!(code, "    fn styles(&self) -> Styles {{ self.styles.clone() }}").unwrap();
        writeln!(code, "}}").unwrap();
    }

    writeln!(code, "struct MyApp {{",).unwrap();
    writeln!(code, "    state: State,").unwrap();
    writeln!(code, "    navigation: Navigation,").unwrap();
    writeln!(code, "    animations: Vec<Animation>,").unwrap();
    writeln!(code, "    rt: Runtime,").unwrap();
    writeln!(code, "    lazy_pages: HashMap<String, Box<dyn Fn() -> Box<dyn Component>>>,").unwrap();
    writeln!(code, "    i18n: I18n,").unwrap();
    writeln!(code, "    dirty_components: HashMap<String, bool>,").unwrap();
    writeln!(code, "    pages: Vec<Rc<RefCell<Box<dyn Component>>>>,").unwrap();
    writeln!(code, "}}").unwrap();

    writeln!(code, "impl MyApp {{",).unwrap();
    writeln!(code, "    fn new() -> Self {{",).unwrap();
    writeln!(code, "        let mut lazy_pages = HashMap::new();").unwrap();
    for page in &ast.pages {
        if let Some(lazy_path) = &page.lazy {
            writeln!(code, "        lazy_pages.insert(\"{}\".to_string(), Box::new(|| Box::new(lazy_load_page(\"{}\"))));", page.route, lazy_path).unwrap();
        }
    }
    writeln!(code, "        let mut app = Self {{",).unwrap();
    writeln!(code, "            state: State::new(),").unwrap();
    writeln!(code, "            navigation: Navigation::new(),").unwrap();
    writeln!(code, "            animations: Vec::new(),").unwrap();
    writeln!(code, "            rt: Runtime::new().unwrap(),").unwrap();
    writeln!(code, "            lazy_pages,").unwrap();
    writeln!(code, "            i18n: I18n::new(I18N.clone()),").unwrap();
    writeln!(code, "            dirty_components: HashMap::new(),").unwrap();
    writeln!(code, "            pages: Vec::new(),").unwrap();
    writeln!(code, "        }};").unwrap();
    writeln!(code, "        let pages = vec![");unwrap();
    for page in &ast.pages {
        writeln!(code, "            Rc::new(RefCell::new(Box::new({} as Box<dyn Component>))),", compile_component(&ParserComponent {
            name: "Page".to_string(),
            styles: page.styles.clone(),
            content: Some(format!("Page: {}", page.name)),
            children: page.children.clone(),
            ..Default::default()
        }, 0, 0, &ast)).unwrap();
    }
    writeln!(code, "        ];").unwrap();
    writeln!(code, "        app.pages = pages;").unwrap();
    writeln!(code, "        app.navigation.set_pages(vec![{}]);", ast.pages.iter().map(|p| format!("Page {{ name: \"{}\".to_string(), route: \"{}\".to_string(), before_enter: {}, before_leave: {}, lazy: {}, styles: Styles {{ props: {:#?} }}, children: vec![] }}", 
        p.name, p.route, p.before_enter.as_ref().map_or("None".to_string(), |v| format!("Some(\"{}\".to_string())", v)), 
        p.before_leave.as_ref().map_or("None".to_string(), |v| format!("Some(\"{}\".to_string())", v)), 
        p.lazy.as_ref().map_or("None".to_string(), |v| format!("Some(\"{}\".to_string())", v)), 
        resolve_styles(&p.styles, &ast.vars))).collect::<Vec<_>>().join(", ")).unwrap();
    writeln!(code, "        app").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "    fn navigate(&mut self, route: &str) {{",).unwrap();
    writeln!(code, "        if let Some(before_leave) = self.navigation.current_page().and_then(|p| p.before_leave.as_ref()) {{",).unwrap();
    writeln!(code, "            self.state.call(before_leave);").unwrap();
    writeln!(code, "        }}").unwrap();
    writeln!(code, "        if let Some(page) = self.navigation.pages.iter().find(|p| p.route == route) {{",).unwrap();
    writeln!(code, "            if let Some(before_enter) = &page.before_enter {{",).unwrap();
    writeln!(code, "                self.state.call(before_enter);").unwrap();
    writeln!(code, "            }}").unwrap();
    writeln!(code, "        }}").unwrap();
    writeln!(code, "        self.navigation.push(route, Some(\"fade\"));").unwrap();
    writeln!(code, "        self.dirty_components.insert(route.to_string(), true);").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "    fn go_back(&mut self) {{",).unwrap();
    writeln!(code, "        if let Some(route) = self.navigation.pop() {{ self.dirty_components.insert(route, true); }}").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "    fn render_ssr(&self, route: &str) -> String {{",).unwrap();
    writeln!(code, "        let mut html = String::new();").unwrap();
    for page in &ast.pages {
        writeln!(code, "        if route == \"{}\" {{", page.route).unwrap();
        writeln!(code, "            html.push_str(&render_to_string(&[");unwrap();
        for comp in &page.children {
            writeln!(code, "                Box::new({}) as Box<dyn Component>,", compile_component(comp, 0, 0, &ast)).unwrap();
        }
        writeln!(code, "            ]));").unwrap();
        writeln!(code, "        }}").unwrap();
    }
    writeln!(code, "        html").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "}}").unwrap();

    writeln!(code, "impl CanvasApp for MyApp {{",).unwrap();
    writeln!(code, "    fn render(&mut self, canvas: &mut Canvas) {{",).unwrap();
    writeln!(code, "        let current_route = self.navigation.current_route();").unwrap();
    writeln!(code, "        if !self.dirty_components.get(current_route).unwrap_or(&true) {{ return; }}").unwrap();
    writeln!(code, "        for page in &self.pages {{",).unwrap();
    writeln!(code, "            let page_ref = page.borrow();").unwrap();
    writeln!(code, "            if page_ref.styles().get(\"route\").unwrap_or(&String::new()) == current_route {{",).unwrap();
    writeln!(code, "                page_ref.render(canvas, &page_ref.styles(), &mut self.animations);").unwrap();
    writeln!(code, "                break;").unwrap();
    writeln!(code, "            }}").unwrap();
    writeln!(code, "        }}").unwrap();
    for page in &ast.pages {
        if page.lazy.is_some() {
            writeln!(code, "        if current_route == \"{}\" {{", page.route).unwrap();
            writeln!(code, "            if let Some(lazy_page) = self.lazy_pages.get(\"{}\") {{", page.route).unwrap();
            writeln!(code, "                let page = lazy_page();").unwrap();
            writeln!(code, "                page.render(canvas, &Styles::new(), &mut self.animations);").unwrap();
            writeln!(code, "            }}").unwrap();
            writeln!(code, "        }}").unwrap();
        }
    }
    writeln!(code, "        if let Some(transition) = self.navigation.transition() {{",).unwrap();
    writeln!(code, "            self.animations.push(Animation::new(transition, 300, Styles::new(), Styles::new()));").unwrap();
    writeln!(code, "        }}").unwrap();
    writeln!(code, "        self.dirty_components.insert(current_route.to_string(), false);").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "    fn on_touch(&mut self, x: i32, y: i32) {{",).unwrap();
    writeln!(code, "        let current_route = self.navigation.current_route();").unwrap();
    writeln!(code, "        for page in &self.pages {{",).unwrap();
    writeln!(code, "            let page_ref = page.borrow();").unwrap();
    writeln!(code, "            if page_ref.styles().get(\"route\").unwrap_or(&String::new()) == current_route {{",).unwrap();
    writeln!(code, "                if let Some(children) = page_ref.children() {{",).unwrap();
    writeln!(code, "                    for child in children {{",).unwrap();
    writeln!(code, "                        let child_ref = child.borrow();").unwrap();
    writeln!(code, "                        let styles = child_ref.styles();").unwrap();
    writeln!(code, "                        let cx = styles.get_px(\"x\", 0);").unwrap();
    writeln!(code, "                        let cy = styles.get_px(\"y\", 0);").unwrap();
    writeln!(code, "                        let cw = styles.get_px(\"width\", 100);").unwrap();
    writeln!(code, "                        let ch = styles.get_px(\"height\", 50);").unwrap();
    writeln!(code, "                        if x >= cx && x < cx + cw && y >= cy && y < cy + ch {{",).unwrap();
    writeln!(code, "                            if let Some(ref on_click) = child_ref.on_click() {{",).unwrap();
    writeln!(code, "                                self.state.call(on_click);").unwrap();
    writeln!(code, "                            }}").unwrap();
    writeln!(code, "                        }}").unwrap();
    writeln!(code, "                    }}").unwrap();
    writeln!(code, "                }}").unwrap();
    writeln!(code, "            }}").unwrap();
    writeln!(code, "        }}").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "}}").unwrap();

    for (_, func) in ast.functions {
        writeln!(code, "{} fn {}({}) -> {} {{", 
            if func.return_type.as_ref().map_or(false, |t| t == "async") { "async" } else { "" },
            func.name,
            func.params.iter().map(|(n, t)| format!("{}: {}", n, match t.as_str() {"string" => "String", "number" => "f64", "bool" => "bool", _ => "String"})).collect::<Vec<_>>().join(", "),
            func.return_type.as_ref().map_or("String", |t| match t.as_str() {"string" => "String", "number" => "f64", "bool" => "bool", _ => "String"}),
        ).unwrap();
        for expr in &func.body {
            match expr {
                Expr::Return(var, value) => writeln!(code, "        let {} = \"{}\".to_string();", var, value).unwrap(),
                Expr::Call(call) => writeln!(code, "        State::new().call(\"{}\");", call).unwrap(),
                Expr::Operation(left, op, right) => writeln!(code, "        let result = {} {} {};", left, op, right).unwrap(),
                Expr::If(cond1, cond2, body) => {
                    writeln!(code, "        if {} == {} {{", cond1, cond2).unwrap();
                    for e in body { writeln!(code, "{}", compile_expr(e)).unwrap(); }
                    writeln!(code, "        }}").unwrap();
                }
                Expr::For(items, item, body) => {
                    writeln!(code, "        for {} in {} {{", item, items).unwrap();
                    for e in body { writeln!(code, "{}", compile_expr(e)).unwrap(); }
                    writeln!(code, "        }}").unwrap();
                }
                Expr::Switch(var, cases) => {
                    writeln!(code, "        match {} {{", var).unwrap();
                    for (value, body) in cases {
                        writeln!(code, "            \"{}\" => {{", value).unwrap();
                        for e in body { writeln!(code, "{}", compile_expr(e)).unwrap(); }
                        writeln!(code, "            }}").unwrap();
                    }
                    writeln!(code, "            _ => {{}}").unwrap();
                    writeln!(code, "        }}").unwrap();
                }
                Expr::Fetch(url, options) => writeln!(code, "        reqwest::get({}).await.unwrap();", url).unwrap(),
            }
        }
        writeln!(code, "        String::new()").unwrap();
        writeln!(code, "    }}").unwrap();
    }

    writeln!(code, "#[cfg(debug_assertions)]").unwrap();
    writeln!(code, "fn main() {{ run_app(MyApp::new()); }}").unwrap();
    writeln!(code, "#[cfg(not(debug_assertions))]").unwrap();
    writeln!(code, "fn main() {{ let app = MyApp::new(); println!(\"{{}}\", app.render_ssr(\"/login\")); }}").unwrap();

    code
}

fn compile_component(comp: &ParserComponent, x: i32, y: i32, ast: &AST) -> String {
    let mut code = String::new();
    if comp.lazy.is_some() {
        writeln!(code, "lazy_load_page(\"{}\")", comp.lazy.as_ref().unwrap()).unwrap();
    } else if ast.components.contains_key(&comp.name) {
        writeln!(code, "{}::new(HashMap::from([", comp.name).unwrap();
        for (prop_name, (prop_type, _)) in &comp.props {
            let value = comp.content.as_ref().unwrap_or(&"".to_string());
            let parsed_value = if value.starts_with("t:") {
                format!("I18N.get(\"{}\").unwrap_or(&String::new()).clone()", value.trim_start_matches("t:").trim_matches('"'))
            } else {
                format!("\"{}\".to_string()", value.trim_matches('"'))
            };
            writeln!(code, "    (\"{}\".to_string(), {}),", prop_name, parsed_value).unwrap();
        }
        writeln!(code, "]))").unwrap();
    } else {
        writeln!(code, "{} {{", comp.name).unwrap();
        writeln!(code, "    name: \"{}\".to_string(),", comp.name).unwrap();
        writeln!(code, "    styles: Styles {{ props: {:#?} }},", resolve_styles(&comp.styles, &ast.vars)).unwrap();
        if let Some(content) = &comp.content { 
            let content_val = if content.starts_with("t:") {
                format!("Reactive::new(I18N.get(\"{}\").unwrap_or(&String::new()).clone())", content.trim_start_matches("t:").trim_matches('"'))
            } else {
                format!("Reactive::new(\"{}\".to_string())", content.trim_matches('"'))
            };
            writeln!(code, "    content: {},", content_val).unwrap();
        }
        if !comp.children.is_empty() {
            writeln!(code, "    children: vec![", comp.name).unwrap();
            for child in &comp.children {
                writeln!(code, "        Rc::new(RefCell::new(Box::new({}))),", compile_component(child, x, y, ast)).unwrap();
            }
            writeln!(code, "    ],").unwrap();
        }
        if let Some(on_click) = &comp.on_click { writeln!(code, "    on_click: Some(\"{}\".to_string()),", on_click).unwrap(); }
        if let Some(on_mount) = &comp.on_mount { writeln!(code, "    on_mount: Some(\"{}\".to_string()),", on_mount).unwrap(); }
        if let Some(on_update) = &comp.on_update { writeln!(code, "    on_update: Some(\"{}\".to_string()),", on_update).unwrap(); }
        if let Some(on_unmount) = &comp.on_unmount { writeln!(code, "    on_unmount: Some(\"{}\".to_string()),", on_unmount).unwrap(); }
        writeln!(code, "    dirty: true,").unwrap();
        writeln!(code, "}}").unwrap();
    }
    code
}

fn resolve_styles(styles: &crate::parser::Styles, vars: &HashMap<String, String>) -> HashMap<String, String> {
    let mut resolved = HashMap::new();
    for (key, value) in &styles.props {
        if value.starts_with('$') {
            if let Some(var_value) = vars.get(&value[1..]) {
                resolved.insert(key.clone(), var_value.clone());
            } else {
                resolved.insert(key.clone(), value.clone());
            }
        } else {
            resolved.insert(key.clone(), value.clone());
        }
    }
    resolved
}

fn compile_expr(expr: &Expr) -> String {
    match expr {
        Expr::Return(var, value) => format!("        let {} = \"{}\".to_string();", var, value),
        Expr::Call(call) => format!("        State::new().call(\"{}\");", call),
        Expr::Operation(left, op, right) => format!("        let result = {} {} {};", left, op, right),
        Expr::If(cond1, cond2, body) => {
            let mut s = format!("        if {} == {} {{\n", cond1, cond2);
            for e in body { s.push_str(&compile_expr(e)); s.push('\n'); }
            s.push_str("        }");
            s
        }
        Expr::For(items, item, body) => {
            let mut s = format!("        for {} in {} {{\n", item, items);
            for e in body { s.push_str(&compile_expr(e)); s.push('\n'); }
            s.push_str("        }");
            s
        }
        Expr::Switch(var, cases) => {
            let mut s = format!("        match {} {{\n", var);
            for (value, body) in cases {
                s.push_str(&format!("            \"{}\" => {{\n", value));
                for e in body { s.push_str(&compile_expr(e)); s.push('\n'); }
                s.push_str("            }\n");
            }
            s.push_str("            _ => {}\n        }");
            s
        }
        Expr::Fetch(url, options) => format!("        reqwest::get({}).await.unwrap();", url),
    }
}