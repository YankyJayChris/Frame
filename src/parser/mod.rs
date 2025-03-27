use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
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
    pub on_mount: Option<String>,
    pub on_update: Option<String>,
    pub on_unmount: Option<String>,
    pub animate: Option<Animation>,
    pub data: Option<String>,
    pub build: Option<(String, Component)>, // (param, body)
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
    let pairs = FrameParser::parse(Rule::file, &project_file)?.next().unwrap();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::vars => {
                for var_pair in pair.into_inner() {
                    if var_pair.as_rule() == Rule::ident {
                        let name = var_pair.as_str().to_string();
                        let value = var_pair.into_inner().next().unwrap().as_str().to_string();
                        ast.vars.insert(name, value);
                    }
                }
            }
            Rule::i18n => {
                for i18n_pair in pair.into_inner() {
                    if i18n_pair.as_rule() == Rule::ident {
                        let key = i18n_pair.as_str().to_string();
                        let value = i18n_pair.into_inner().next().unwrap().as_str().to_string();
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
                        Rule::path => path = Some(import_part.as_str().to_string()),
                        _ => {}
                    }
                }
                let imports = ast.imports.entry(path.unwrap()).or_insert_with(Vec::new);
                imports.push((ident.unwrap(), alias));
            }
            Rule::page => {
                let mut page = Page {
                    styles: Styles::default(),
                    children: Vec::new(),
                    ..Default::default()
                };
                for page_part in pair.into_inner() {
                    match page_part.as_rule() {
                        Rule::string => {
                            if page.name.is_empty() {
                                page.name = page_part.as_str().trim_matches('"').to_string();
                            } else {
                                page.route = page_part.as_str().trim_matches('"').to_string();
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
                    params: Vec::new(),
                    body: Vec::new(),
                    ..Default::default()
                };
                for fn_part in pair.into_inner() {
                    match fn_part.as_rule() {
                        Rule::ident => {
                            if func.name.is_empty() {
                                func.name = fn_part.as_str().to_string();
                            } else if func.return_type.is_none() {
                                func.return_type = Some(fn_part.as_str().to_string());
                            } else {
                                func.params.push((fn_part.as_str().to_string(), fn_part.into_inner().next().unwrap().as_str().to_string()));
                            }
                        }
                        Rule::expr => func.body.push(parse_expr(fn_part)),
                        _ => {}
                    }
                }
                ast.functions.insert(func.name.clone(), func);
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
            let key = style_pair.as_str().to_string();
            let value = style_pair.into_inner().next().unwrap().as_str().to_string();
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
            Rule::children => comp.children = comp_part.into_inner().map(parse_component).collect(),
            Rule::on_click => comp.on_click = Some(parse_call(comp_part.into_inner().next().unwrap())),
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
                        Rule::styles => {
                            if anim.from.props.is_empty() {
                                anim.from = parse_styles(anim_part);
                            } else {
                                anim.to = parse_styles(anim_part);
                            }
                        }
                        Rule::number => anim.duration = anim_part.as_str().parse().unwrap(),
                        Rule::string => anim.easing = anim_part.as_str().trim_matches('"').to_string(),
                        _ => {}
                    }
                }
                comp.animate = Some(anim);
            }
            Rule::props => {
                for prop_pair in comp_part.into_inner() {
                    if prop_pair.as_rule() == Rule::ident {
                        let name = prop_pair.as_str().to_string();
                        let type_ = prop_pair.into_inner().next().unwrap().as_str().to_string();
                        let required = prop_pair.into_inner().any(|p| p.as_rule() == Rule::string && p.as_str() == "required");
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
                        Rule::ident => {
                            if param.is_empty() {
                                param = build_part.as_str().to_string();
                            }
                        }
                        Rule::component => body = Some(parse_component(build_part)),
                        _ => {}
                    }
                }
                comp.build = Some((param, body.unwrap()));
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
            let first = inner.next().unwrap();
            match first.as_rule() {
                Rule::return_stmt => {
                    let var = first.into_inner().next().unwrap().as_str().to_string();
                    let value = inner.next().unwrap().as_str().to_string();
                    Expr::Return(var, value)
                }
                Rule::call => Expr::Call(parse_call(first)),
                Rule::operation => {
                    let left = first.as_str().to_string();
                    let op = inner.next().unwrap().as_str().to_string();
                    let right = inner.next().unwrap().as_str().to_string();
                    Expr::Operation(left, op, right)
                }
                Rule::if_stmt => {
                    let cond1 = first.into_inner().next().unwrap().as_str().to_string();
                    let cond2 = inner.next().unwrap().as_str().to_string();
                    let body = inner.next().unwrap().into_inner().map(parse_expr).collect();
                    Expr::If(cond1, cond2, body)
                }
                Rule::for_stmt => {
                    let items = first.into_inner().next().unwrap().as_str().to_string();
                    let item = inner.next().unwrap().as_str().to_string();
                    let body = inner.next().unwrap().into_inner().map(parse_expr).collect();
                    Expr::For(items, item, body)
                }
                Rule::switch_stmt => {
                    let var = first.into_inner().next().unwrap().as_str().to_string();
                    let cases = inner.map(|case| {
                        let value = case.into_inner().next().unwrap().as_str().trim_matches('"').to_string();
                        let body = case.into_inner().map(parse_expr).collect();
                        (value, body)
                    }).collect();
                    Expr::Switch(var, cases)
                }
                Rule::fetch => {
                    let url = first.into_inner().next().unwrap().as_str().to_string();
                    let options = inner.next().unwrap().as_str().to_string();
                    Expr::Fetch(url, options)
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

fn parse_call(pair: pest::iterators::Pair<Rule>) -> String {
    let mut call_str = String::new();
    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::ident => call_str.push_str(part.as_str()),
            Rule::arg => call_str.push_str(&format!("({})", part.as_str())),
            _ => {}
        }
    }
    call_str
}