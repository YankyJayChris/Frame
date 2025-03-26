use crate::parser::{AST, Component, Expr};
use std::fmt::Write;

pub fn compile(ast: AST) -> String {
    let mut code = String::new();

    writeln!(code, "use frame_core::runtime::{{run_app, CanvasApp, State, Navigation, Animation}};").unwrap();
    writeln!(code, "use frame_core::components::*;").unwrap();
    writeln!(code, "use frame_core::styles::Styles;").unwrap();
    writeln!(code, "use std::collections::HashMap;").unwrap();
    writeln!(code, "use tokio::runtime::Runtime;").unwrap();

    writeln!(code, "fn main() {{ run_app(MyApp::new()); }}").unwrap();

    writeln!(code, "struct MyApp {{").unwrap();
    writeln!(code, "    state: State,").unwrap();
    writeln!(code, "    navigation: Navigation,").unwrap();
    writeln!(code, "    animations: Vec<Animation>,").unwrap();
    writeln!(code, "    rt: Runtime,").unwrap();
    writeln!(code, "}}").unwrap();

    writeln!(code, "impl MyApp {{").unwrap();
    writeln!(code, "    fn new() -> Self {{").unwrap();
    writeln!(code, "        Self {{").unwrap();
    writeln!(code, "            state: State::new(),").unwrap();
    writeln!(code, "            navigation: Navigation::new(),").unwrap();
    writeln!(code, "            animations: Vec::new(),").unwrap();
    writeln!(code, "            rt: Runtime::new().unwrap(),").unwrap();
    writeln!(code, "        }}").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "}}").unwrap();

    writeln!(code, "impl CanvasApp for MyApp {{").unwrap();
    writeln!(code, "    fn render(&mut self, canvas: &mut frame_core::runtime::Canvas) {{").unwrap();
    writeln!(code, "        let current_route = self.navigation.current_route();").unwrap();
    for page in ast.pages {
        writeln!(code, "        if current_route == \"{}\" {{", page.route).unwrap();
        writeln!(code, "            let layout = Styles::layout(&[");unwrap();
        for comp in &page.children {
            writeln!(code, "                {} {{", comp.name).unwrap();
            writeln!(code, "                    styles: Styles {{ props: {:#?} }},", comp.styles.props).unwrap();
            if let Some(content) = &comp.content { writeln!(code, "                    content: \"{}\".to_string(),", content).unwrap(); }
            if let Some(data) = &comp.data { writeln!(code, "                    data: self.state.get(\"{}\").unwrap_or(&vec![]),", data).unwrap(); }
            if let Some(on_click) = &comp.on_click { writeln!(code, "                    on_click: Box::new(|| self.state.call(\"{}\")),", on_click).unwrap(); }
            if let Some((param, build)) = &comp.build { 
                writeln!(code, "                    build: Box::new(|{}| {{ {} }}),", param, compile_component(build, 0, 0, &ast)).unwrap();
            }
            if let Some(value) = &comp.value { writeln!(code, "                    value: \"{}\".to_string(),", value).unwrap(); }
            if !comp.options.is_empty() { writeln!(code, "                    options: vec![{}],", comp.options.iter().map(|o| format!("\"{}\".to_string()", o)).collect::<Vec<_>>().join(", ")).unwrap(); }
            if let Some(on_change) = &comp.on_change { writeln!(code, "                    on_change: Box::new(|v| self.state.call(\"{}\")),", on_change).unwrap(); }
            if let Some(on_select) = &comp.on_select { writeln!(code, "                    on_select: Box::new(|i| self.state.call(\"select:{}\")),", on_select).unwrap(); }
            if let Some(on_submit) = &comp.on_submit { writeln!(code, "                    on_submit: Box::new(|| self.state.call(\"{}\")),", on_submit).unwrap(); }
            if let Some(validation) = &comp.validation { writeln!(code, "                    validation: \"{}\".to_string(),", validation).unwrap(); }
            writeln!(code, "                }},").unwrap();
        }
        writeln!(code, "            ]);").unwrap();
        writeln!(code, "            layout.render(canvas, &mut self.animations);").unwrap();
        writeln!(code, "        }}").unwrap();
    }
    writeln!(code, "    }}").unwrap();
    writeln!(code, "}}").unwrap();

    for (_, func) in ast.functions {
        writeln!(code, "{} fn {}({}) -> String {{", 
            if func.is_async { "async" } else { "" },
            func.name,
            func.params.iter().map(|(n, t)| format!("{}: {}", n, t)).collect::<Vec<_>>().join(", "),
        ).unwrap();
        for expr in &func.body {
            match expr {
                Expr::Return(var, value) => writeln!(code, "        let {} = \"{}\";", var, value).unwrap(),
                Expr::Call(call) => writeln!(code, "        self.state.call(\"{}\");", call).unwrap(),
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
            }
        }
        writeln!(code, "        String::new()").unwrap();
        writeln!(code, "    }}").unwrap();
    }

    code
}

fn compile_component(comp: &Component, x: i32, y: i32, ast: &AST) -> String {
    let mut code = String::new();
    match comp.name.as_str() {
        "Card" => {
            if let Some(card) = ast.components.get("Card") {
                writeln!(code, "Card {{ content: \"post\".to_string(), styles: Styles {{ props: {:#?} }} }}", comp.styles.props).unwrap();
            }
        }
        _ => writeln!(code, "{} {{ styles: Styles {{ props: {:#?} }} }}", comp.name, comp.styles.props).unwrap(),
    }
    code
}

fn compile_expr(expr: &Expr) -> String {
    match expr {
        Expr::Return(var, value) => format!("            let {} = \"{}\";", var, value),
        Expr::Call(call) => format!("            self.state.call(\"{}\");", call),
        Expr::Operation(left, op, right) => format!("            let result = {} {} {};", left, op, right),
        Expr::If(cond1, cond2, body) => {
            let mut s = format!("            if {} == {} {{\n", cond1, cond2);
            for e in body { s.push_str(&compile_expr(e)); s.push('\n'); }
            s.push_str("            }");
            s
        }
        Expr::For(items, item, body) => {
            let mut s = format!("            for {} in {} {{\n", item, items);
            for e in body { s.push_str(&compile_expr(e)); s.push('\n'); }
            s.push_str("            }");
            s
        }
        Expr::Switch(var, cases) => {
            let mut s = format!("            match {} {{\n", var);
            for (value, body) in cases {
                s.push_str(&format!("                \"{}\" => {{\n", value));
                for e in body { s.push_str(&compile_expr(e)); s.push('\n'); }
                s.push_str("                }\n");
            }
            s.push_str("                _ => {}\n            }");
            s
        }
    }
}