use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(pest::Parser)]
#[grammar = "frame.pest"]
pub struct FrameParser;

#[derive(Debug, Default)]
pub struct AST {
    pub vars: HashMap<String, String>,
    pub i18n: HashMap<String, String>,
    pub imports: HashMap<String, Vec<(String, Option<String>)>>, // path -> (ident, alias)
    pub pages: Vec<Page>,
    pub components: HashMap<String, Component>,
    pub functions: HashMap<String, Function>,
    pub tests: Vec<TestSuite>,
}

#[derive(Debug)]
pub struct Page {
    pub name: String,
    pub route: String,
    pub before_enter: Option<String>,
    pub before_leave: Option<String>,
    pub lazy: Option<String>,
    pub styles: Styles,
    pub children: Vec<Component>,
}

#[derive(Debug, Default)]
pub struct Component {
    pub name: String,
    pub props: HashMap<String, (String, bool)>, // (type, required)
    pub styles: Styles,
    pub content: Option<String>,
    pub children: Vec<Component>,
    pub on_click: Option<String>,
    pub on_touch_start: Option<String>,         // Added
    pub on_touch_move: Option<String>,          // Added
    pub on_touch_end: Option<String>,           // Added
    pub on_touch_cancel: Option<String>,        // Added
    pub on_mount: Option<String>,
    pub on_update: Option<String>,
    pub on_unmount: Option<String>,
    pub animate: Option<Animation>,
    pub data: Option<String>,
    pub build: Option<(String, Component)>,     // (param, body)
}

#[derive(Debug, Default)]
pub struct Styles {
    pub props: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Animation {
    pub kind: String,
    pub duration: u32,
    pub from: Styles,
    pub to: Styles,
    pub easing: String,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, type)
    pub return_type: Option<String>,
    pub body: Vec<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Return(String, String),
    Call(String),
    Operation(String, String, String),
    If(String, String, Vec<Expr>),
    For(String, String, Vec<Expr>),
    Switch(String, Vec<(String, Vec<Expr>)>),
    Fetch(String, String),
}

#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    pub cases: Vec<TestCase>,
}

#[derive(Debug)]
pub struct TestCase {
    pub name: String,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug)]
pub struct Assertion {
    pub target: String,
    pub method: String,
    pub value: String,
}

pub fn parse_project(dir: &str) -> Result<AST, Box<dyn std::error::Error>> {
    let mut ast = AST::default();
    let project_file = std::fs::read_to_string(format!("{}/src/project.fr", dir))?;
    let pairs = FrameParser::parse(Rule::file, &project_file)?.next().ok_or("Empty file")?;

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::vars => {
                for var_pair in pair.into_inner() {
                    if var_pair.as_rule() == Rule::ident {
                        let mut inner = var_pair.into_inner();
                        let name = inner.next().ok_or("Missing var name")?.as_str().to_string();
                        let value = inner.next().ok_or("Missing var value")?.as_str().to_string();
                        ast.vars.insert(name, value);
                    }
                }
            }
            Rule::i18n => {
                for i18n_pair in pair.into_inner() {
                    if i18n_pair.as_rule() == Rule::ident {
                        let mut inner = i18n_pair.into_inner();
                        let key = inner.next().ok_or("Missing i18n key")?.as_str().to_string();
                        let value = inner.next().ok_or("Missing i18n value")?.as_str().to_string();
                        ast.i18n.insert(key, value);
                    }
                }
            }
            Rule::import => {
                let mut ident = None;
                let mut alias = None;
                let mut path = None;
                for import_part in pair.into_inner() {
                    match import_part.as_rule() {
                        Rule::ident => {
                            if ident.is_none() {
                                ident = Some(import_part.as_str().to_string());
                            } else {
                                alias = Some(import_part.as_str().to_string());
                            }
                        }
                        Rule::path => path = Some(import_part.as_str().trim_matches('"').to_string()),
                        _ => {}
                    }
                }
                let imports = ast.imports.entry(path.ok_or("Missing import path")?).or_insert_with(Vec::new);
                imports.push((ident.ok_or("Missing import ident")?, alias));
            }
            Rule::page => {
                let mut page = Page {
                    name: String::new(),
                    route: String::new(),
                    before_enter: None,
                    before_leave: None,
                    lazy: None,
                    styles: Styles::default(),
                    children: Vec::new(),
                };
                for page_part in pair.into_inner() {
                    match page_part.as_rule() {
                        Rule::string => {
                            if page.name.is_empty() {
                                page.name = page_part.as_str().trim_matches('"').to_string();
                            } else if page.route.is_empty() {
                                page.route = page_part.as_str().trim_matches('"').to_string();
                            } else {
                                page.lazy = Some(page_part.as_str().trim_matches('"').to_string());
                            }
                        }
                        Rule::call => {
                            let call_str = parse_call(page_part);
                            if page.before_enter.is_none() {
                                page.before_enter = Some(call_str);
                            } else {
                                page.before_leave = Some(call_str);
                            }
                        }
                        Rule::styles => page.styles = parse_styles(page_part),
                        Rule::component => page.children.push(parse_component(page_part)),
                        _ => {}
                    }
                }
                ast.pages.push(page);
            }
            Rule::fn_def => {
                let mut func = Function {
                    name: String::new(),
                    params: Vec::new(),
                    return_type: None,
                    body: Vec::new(),
                };
                for fn_part in pair.into_inner() {
                    match fn_part.as_rule() {
                        Rule::ident => {
                            if func.name.is_empty() {
                                func.name = fn_part.as_str().to_string();
                            } else {
                                let mut inner = fn_part.into_inner();
                                let param_name = inner.next().ok_or("Missing param name")?.as_str().to_string();
                                let param_type = inner.next().ok_or("Missing param type")?.as_str().to_string();
                                func.params.push((param_name, param_type));
                            }
                        }
                        Rule::type_name => func.return_type = Some(fn_part.as_str().to_string()),
                        Rule::expr => func.body.push(parse_expr(fn_part)),
                        _ => {}
                    }
                }
                ast.functions.insert(func.name.clone(), func);
            }
            Rule::component => {
                let comp = parse_component(pair);
                ast.components.insert(comp.name.clone(), comp);
            }
            Rule::test_suite => {
                let mut suite = TestSuite {
                    name: String::new(),
                    cases: Vec::new(),
                };
                for suite_part in pair.into_inner() {
                    match suite_part.as_rule() {
                        Rule::string => suite.name = suite_part.as_str().trim_matches('"').to_string(),
                        Rule::test_case => {
                            let mut case = TestCase {
                                name: String::new(),
                                assertions: Vec::new(),
                            };
                            for case_part in suite_part.into_inner() {
                                match case_part.as_rule() {
                                    Rule::string => case.name = case_part.as_str().trim_matches('"').to_string(),
                                    Rule::assertion => {
                                        let mut assertion = Assertion {
                                            target: String::new(),
                                            method: String::new(),
                                            value: String::new(),
                                        };
                                        for assert_part in case_part.into_inner() {
                                            match assert_part.as_rule() {
                                                Rule::ident => {
                                                    if assertion.target.is_empty() {
                                                        assertion.target = assert_part.as_str().to_string();
                                                    } else {
                                                        assertion.method = assert_part.as_str().to_string();
                                                    }
                                                }
                                                Rule::string | Rule::number => assertion.value = assert_part.as_str().to_string(),
                                                _ => {}
                                            }
                                        }
                                        case.assertions.push(assertion);
                                    }
                                    _ => {}
                                }
                            }
                            suite.cases.push(case);
                        }
                        _ => {}
                    }
                }
                ast.tests.push(suite);
            }
            _ => {}
        }
    }
    Ok(ast)
}

fn parse_styles(pair: pest::iterators::Pair<Rule>) -> Styles {
    let mut styles = Styles::default();
    for style_pair in pair.into_inner() {
        if style_pair.as_rule() == Rule::ident {
            let mut inner = style_pair.into_inner();
            let key = inner.next().ok_or("Missing style key").map(|p| p.as_str().to_string()).unwrap();
            let value = inner.next().ok_or("Missing style value").map(|p| p.as_str().to_string()).unwrap();
            styles.props.insert(key, value);
        }
    }
    styles
}

fn parse_component(pair: pest::iterators::Pair<Rule>) -> Component {
    let mut comp = Component::default();
    for comp_part in pair.into_inner() {
        match comp_part.as_rule() {
            Rule::ident => comp.name = comp_part.as_str().to_string(),
            Rule::styles => comp.styles = parse_styles(comp_part),
            Rule::content => comp.content = Some(comp_part.into_inner().next().unwrap().as_str().to_string()),
            Rule::children => comp.children = comp_part.into_inner().filter(|p| p.as_rule() == Rule::component).map(parse_component).collect(),
            Rule::on_click => comp.on_click = Some(parse_call(comp_part.into_inner().next().unwrap())),
            Rule::on_touch_start => comp.on_touch_start = Some(parse_call(comp_part.into_inner().next().unwrap())), // Added
            Rule::on_touch_move => comp.on_touch_move = Some(parse_call(comp_part.into_inner().next().unwrap())),   // Added
            Rule::on_touch_end => comp.on_touch_end = Some(parse_call(comp_part.into_inner().next().unwrap())),     // Added
            Rule::on_touch_cancel => comp.on_touch_cancel = Some(parse_call(comp_part.into_inner().next().unwrap())), // Added
            Rule::on_mount => comp.on_mount = Some(parse_call(comp_part.into_inner().next().unwrap())),
            Rule::on_update => comp.on_update = Some(parse_call(comp_part.into_inner().next().unwrap())),
            Rule::on_unmount => comp.on_unmount = Some(parse_call(comp_part.into_inner().next().unwrap())),
            Rule::animate => {
                let mut anim = Animation {
                    kind: "default".to_string(),
                    duration: 0,
                    from: Styles::default(),
                    to: Styles::default(),
                    easing: "linear".to_string(),
                };
                for anim_part in comp_part.into_inner() {
                    match anim_part.as_rule() {
                        Rule::ident => anim.kind = anim_part.as_str().to_string(),
                        Rule::number => anim.duration = anim_part.as_str().parse().unwrap_or(0),
                        Rule::styles => {
                            if anim.from.props.is_empty() {
                                anim.from = parse_styles(anim_part);
                            } else {
                                anim.to = parse_styles(anim_part);
                            }
                        }
                        Rule::string => anim.easing = anim_part.as_str().trim_matches('"').to_string(),
                        _ => {}
                    }
                }
                comp.animate = Some(anim);
            }
            Rule::props => {
                for prop_pair in comp_part.into_inner() {
                    if prop_pair.as_rule() == Rule::ident {
                        let mut inner = prop_pair.into_inner();
                        let name = inner.next().ok_or("Missing prop name").map(|p| p.as_str().to_string()).unwrap();
                        let type_ = inner.next().ok_or("Missing prop type").map(|p| p.as_str().to_string()).unwrap();
                        let required = inner.any(|p| p.as_rule() == Rule::string && p.as_str() == "required");
                        comp.props.insert(name, (type_, required));
                    }
                }
            }
            Rule::data => comp.data = Some(comp_part.into_inner().next().unwrap().as_str().to_string()),
            Rule::build => {
                let mut param = String::new();
                let mut body = None;
                for build_part in comp_part.into_inner() {
                    match build_part.as_rule() {
                        Rule::ident => param = build_part.as_str().to_string(),
                        Rule::component => body = Some(parse_component(build_part)),
                        _ => {}
                    }
                }
                comp.build = Some((param, body.unwrap_or_default()));
            }
            _ => {}
        }
    }
    comp
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => {
            let mut inner = pair.into_inner();
            let first = inner.next().ok_or("Missing expr part").unwrap();
            match first.as_rule() {
                Rule::return_stmt => {
                    let mut inner = first.into_inner();
                    let var = inner.next().ok_or("Missing return var").map(|p| p.as_str().to_string()).unwrap();
                    let value = inner.next().ok_or("Missing return value").map(|p| p.as_str().to_string()).unwrap();
                    Expr::Return(var, value)
                }
                Rule::call => Expr::Call(parse_call(first)),
                Rule::operation => {
                    let mut inner = first.into_inner();
                    let left = inner.next().ok_or("Missing left operand").map(|p| p.as_str().to_string()).unwrap();
                    let op = inner.next().ok_or("Missing operator").map(|p| p.as_str().to_string()).unwrap();
                    let right = inner.next().ok_or("Missing right operand").map(|p| p.as_str().to_string()).unwrap();
                    Expr::Operation(left, op, right)
                }
                Rule::if_stmt => {
                    let mut inner = first.into_inner();
                    let cond1 = inner.next().ok_or("Missing if condition 1").map(|p| p.as_str().to_string()).unwrap();
                    let cond2 = inner.next().ok_or("Missing if condition 2").map(|p| p.as_str().to_string()).unwrap();
                    let body = inner.next().ok_or("Missing if body").map(|p| p.into_inner().map(parse_expr).collect()).unwrap();
                    Expr::If(cond1, cond2, body)
                }
                Rule::for_stmt => {
                    let mut inner = first.into_inner();
                    let items = inner.next().ok_or("Missing for items").map(|p| p.as_str().to_string()).unwrap();
                    let item = inner.next().ok_or("Missing for item").map(|p| p.as_str().to_string()).unwrap();
                    let body = inner.next().ok_or("Missing for body").map(|p| p.into_inner().map(parse_expr).collect()).unwrap();
                    Expr::For(items, item, body)
                }
                Rule::switch_stmt => {
                    let mut inner = first.into_inner();
                    let var = inner.next().ok_or("Missing switch var").map(|p| p.as_str().to_string()).unwrap();
                    let cases = inner.map(|case| {
                        let mut case_inner = case.into_inner();
                        let value = case_inner.next().ok_or("Missing case value").map(|p| p.as_str().trim_matches('"').to_string()).unwrap();
                        let body = case_inner.map(parse_expr).collect();
                        Ok((value, body))
                    }).collect::<Result<Vec<_>, &'static str>>()?;
                    Expr::Switch(var, cases)
                }
                Rule::fetch => {
                    let mut inner = first.into_inner();
                    let url = inner.next().ok_or("Missing fetch url").map(|p| p.as_str().to_string()).unwrap();
                    let options = inner.next().ok_or("Missing fetch options").map(|p| p.as_str().to_string()).unwrap();
                    Expr::Fetch(url, options)
                }
                _ => return Err("Unknown expression type").unwrap(),
            }
        }
        _ => return Err("Expected expression").unwrap(),
    }
}

fn parse_call(pair: pest::iterators::Pair<Rule>) -> String {
    let mut call_str = String::new();
    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::ident => call_str.push_str(part.as_str()),
            Rule::arg => call_str.push_str(&format!("({})", part.into_inner().map(|p| p.as_str()).collect::<Vec<_>>().join(","))),
            _ => {}
        }
    }
    call_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vars() {
        let input = r#":vars { $primary: "#007BFF" }";
        let mut ast = AST::default();
        let pairs = FrameParser::parse(Rule::file, input).unwrap().next().unwrap();
        for pair in pairs.into_inner() {
            if pair.as_rule() == Rule::vars {
                for var_pair in pair.into_inner() {
                    if var_pair.as_rule() == Rule::ident {
                        let mut inner = var_pair.into_inner();
                        let name = inner.next().unwrap().as_str().to_string();
                        let value = inner.next().unwrap().as_str().to_string();
                        ast.vars.insert(name, value);
                    }
                }
            }
        }
        assert_eq!(ast.vars.get("$primary"), Some(&"#007BFF".to_string()));
    }
}