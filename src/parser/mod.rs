use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct FrameParser;

#[derive(Debug)]
pub struct AST {
    pub pages: Vec<Page>,
    pub functions: HashMap<String, Function>,
    pub components: HashMap<String, ComponentDef>,
    pub tests: Vec<TestSuite>,
}

#[derive(Debug)]
pub struct Page {
    pub name: String,
    pub route: String,
    pub styles: Styles,
    pub children: Vec<Component>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub is_async: bool,
    pub params: Vec<(String, String)>,
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
}

#[derive(Debug)]
pub struct ComponentDef {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub body: Component,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub styles: Styles,
    pub content: Option<String>,
    pub children: Vec<Component>,
    pub on_click: Option<String>,
    pub data: Option<String>,
    pub build: Option<(String, Component)>,
    pub direction: Option<String>,
    pub scroll: Option<String>,
    pub current: Option<i32>,
    pub items: Vec<Item>,
    pub menu_icon: Option<(String, String)>,
    pub actions: Vec<Component>,
    pub src: Option<String>,
    pub value: Option<String>,
    pub options: Vec<String>,
    pub on_change: Option<String>,
    pub on_select: Option<i32>,
    pub on_submit: Option<String>,
    pub validation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub content: String,
    pub icon: String,
    pub on_click: String,
    pub styles: Styles,
}

#[derive(Debug, Clone, Default)]
pub struct Styles {
    pub props: HashMap<String, String>,
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
    pub expected: String,
}

pub fn parse_project(dir: &str) -> Result<AST, Box<pest::error::Error<Rule>>> {
    let mut ast = AST {
        pages: Vec::new(),
        functions: HashMap::new(),
        components: HashMap::new(),
        tests: Vec::new(),
    };

    let root_content = fs::read_to_string(format!("{}/src/project.fr", dir))
        .map_err(|e| pest::error::Error::new_from_span(pest::error::ErrorVariant::CustomError { message: e.to_string() }, pest::Span::new("", 0, 0).unwrap()))?;
    let root_pairs = FrameParser::parse(Rule::file, &root_content)?;

    for pair in root_pairs {
        match pair.as_rule() {
            Rule::page => ast.pages.push(parse_page(pair)),
            Rule::fn_def => {
                let func = parse_function(pair);
                ast.functions.insert(func.name.clone(), func);
            }
            _ => {}
        }
    }

    let comp_dir = format!("{}/src/component", dir);
    if Path::new(&comp_dir).exists() {
        for entry in fs::read_dir(&comp_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |ext| ext == "fr") {
                let content = fs::read_to_string(&path)?;
                let pairs = FrameParser::parse(Rule::component, &content)?;
                for pair in pairs {
                    let comp = parse_component(pair);
                    ast.components.insert(comp.name.clone(), ComponentDef {
                        name: comp.name.clone(),
                        params: vec![("post".to_string(), "object".to_string())],
                        body: comp,
                    });
                }
            }
        }
    }

    let test_dir = format!("{}/src/tests", dir);
    if Path::new(&test_dir).exists() {
        for entry in fs::read_dir(&test_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |ext| ext == "fr") {
                let content = fs::read_to_string(&path)?;
                let pairs = FrameParser::parse(Rule::test_file, &content)?;
                for pair in pairs {
                    if pair.as_rule() == Rule::test_suite {
                        ast.tests.push(parse_test_suite(pair));
                    }
                }
            }
        }
    }

    Ok(ast)
}

fn parse_page(pair: pest::iterators::Pair<Rule>) -> Page {
    let mut page = Page {
        name: "".to_string(),
        route: "".to_string(),
        styles: Styles::default(),
        children: Vec::new(),
    };
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string => {
                if page.name.is_empty() { page.name = inner.as_str().trim_matches('"').to_string(); }
                else { page.route = inner.as_str().trim_matches('"').to_string(); }
            }
            Rule::styles => page.styles = parse_styles(inner),
            Rule::component => page.children.push(parse_component(inner)),
            _ => {}
        }
    }
    page
}

fn parse_function(pair: pest::iterators::Pair<Rule>) -> Function {
    let mut func = Function {
        name: "".to_string(),
        is_async: false,
        params: Vec::new(),
        body: Vec::new(),
    };
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => func.name = inner.as_str().to_string(),
            Rule::async => func.is_async = true,
            Rule::param => {
                let mut param_name = "";
                let mut param_type = "";
                for p in inner.into_inner() {
                    if p.as_rule() == Rule::ident {
                        if param_name.is_empty() { param_name = p.as_str(); }
                        else { param_type = p.as_str(); }
                    }
                }
                func.params.push((param_name.to_string(), param_type.to_string()));
            }
            Rule::expr => func.body.push(parse_expr(inner)),
            _ => {}
        }
    }
    func
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::if_stmt => {
            let mut cond1 = "";
            let mut cond2 = "";
            let mut body = Vec::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::ident => {
                        if cond1.is_empty() { cond1 = inner.as_str(); }
                        else { cond2 = inner.as_str(); }
                    }
                    Rule::expr => body.push(parse_expr(inner)),
                    _ => {}
                }
            }
            Expr::If(cond1.to_string(), cond2.to_string(), body)
        }
        Rule::for_stmt => {
            let mut items = "";
            let mut item = "";
            let mut body = Vec::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::ident => {
                        if items.is_empty() { items = inner.as_str(); }
                        else { item = inner.as_str(); }
                    }
                    Rule::expr => body.push(parse_expr(inner)),
                    _ => {}
                }
            }
            Expr::For(items.to_string(), item.to_string(), body)
        }
        Rule::switch_stmt => {
            let mut var = "";
            let mut cases = Vec::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::ident => var = inner.as_str(),
                    Rule::case_stmt => {
                        let mut value = "";
                        let mut body = Vec::new();
                        for case_inner in inner.into_inner() {
                            match case_inner.as_rule() {
                                Rule::string => value = case_inner.as_str().trim_matches('"'),
                                Rule::expr => body.push(parse_expr(case_inner)),
                                _ => {}
                            }
                        }
                        cases.push((value.to_string(), body));
                    }
                    _ => {}
                }
            }
            Expr::Switch(var.to_string(), cases)
        }
        Rule::call => Expr::Call(pair.as_str().to_string()),
        Rule::expr => {
            let mut parts = pair.into_inner();
            if pair.as_str().contains("return") {
                let var = parts.next().unwrap().as_str();
                let value = parts.nth(1).unwrap().as_str();
                Expr::Return(var.to_string(), value.to_string())
            } else {
                let left = parts.next().unwrap().as_str();
                let op = parts.next().unwrap().as_str();
                let right = parts.next().unwrap().as_str();
                Expr::Operation(left.to_string(), op.to_string(), right.to_string())
            }
        }
        _ => unreachable!(),
    }
}

fn parse_component(pair: pest::iterators::Pair<Rule>) -> Component {
    let mut comp = Component {
        name: pair.as_str().split(':').next().unwrap().to_string(),
        styles: Styles::default(),
        content: None,
        children: Vec::new(),
        on_click: None,
        data: None,
        build: None,
        direction: None,
        scroll: None,
        current: None,
        items: Vec::new(),
        menu_icon: None,
        actions: Vec::new(),
        src: None,
        value: None,
        options: Vec::new(),
        on_change: None,
        on_select: None,
        on_submit: None,
        validation: None,
    };
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::styles => comp.styles = parse_styles(inner),
            Rule::content => comp.content = Some(inner.as_str().to_string()),
            Rule::component => comp.children.push(parse_component(inner)),
            Rule::on_click => comp.on_click = Some(inner.as_str().to_string()),
            Rule::data => comp.data = Some(inner.as_str().to_string()),
            Rule::build => {
                let mut param = "";
                let mut body = None;
                for b in inner.into_inner() {
                    if b.as_rule() == Rule::ident { param = b.as_str(); }
                    if b.as_rule() == Rule::component { body = Some(parse_component(b)); }
                }
                comp.build = Some((param.to_string(), body.unwrap()));
            }
            Rule::direction => comp.direction = Some(inner.as_str().trim_matches('"').to_string()),
            Rule::scroll => comp.scroll = Some(inner.as_str().trim_matches('"').to_string()),
            Rule::current => comp.current = Some(inner.as_str().parse().unwrap()),
            Rule::item => comp.items.push(parse_item(inner)),
            Rule::menu => {
                let mut icon = "";
                let mut click = "";
                for m in inner.into_inner() {
                    if m.as_rule() == Rule::string { icon = m.as_str().trim_matches('"'); }
                    if m.as_rule() == Rule::call { click = m.as_str(); }
                }
                comp.menu_icon = Some((icon.to_string(), click.to_string()));
            }
            Rule::actions => {
                for a in inner.into_inner() {
                    comp.actions.push(parse_component(a));
                }
            }
            Rule::src => comp.src = Some(inner.as_str().trim_matches('"').to_string()),
            Rule::value => comp.value = Some(inner.as_str().trim_matches('"').to_string()),
            Rule::options => comp.options = inner.into_inner().map(|o| o.as_str().trim_matches('"').to_string()).collect(),
            Rule::on_change => comp.on_change = Some(inner.as_str().to_string()),
            Rule::on_select => comp.on_select = Some(inner.as_str().parse().unwrap()),
            Rule::on_submit => comp.on_submit = Some(inner.as_str().to_string()),
            Rule::validation => comp.validation = Some(inner.as_str().trim_matches('"').to_string()),
            _ => {}
        }
    }
    comp
}

fn parse_item(pair: pest::iterators::Pair<Rule>) -> Item {
    let mut item = Item {
        content: "".to_string(),
        icon: "".to_string(),
        on_click: "".to_string(),
        styles: Styles::default(),
    };
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::content => item.content = inner.as_str().trim_matches('"').to_string(),
            Rule::icon => item.icon = inner.as_str().trim_matches('"').to_string(),
            Rule::on_click => item.on_click = inner.as_str().to_string(),
            Rule::styles => item.styles = parse_styles(inner),
            _ => {}
        }
    }
    item
}

fn parse_styles(pair: pest::iterators::Pair<Rule>) -> Styles {
    let mut styles = Styles::default();
    for style in pair.into_inner() {
        let mut key = "";
        let mut value = "";
        for prop in style.into_inner() {
            if prop.as_rule() == Rule::ident { key = prop.as_str(); }
            else { value = prop.as_str(); }
        }
        styles.props.insert(key.to_string(), value.to_string());
    }
    styles
}

fn parse_test_suite(pair: pest::iterators::Pair<Rule>) -> TestSuite {
    let mut suite = TestSuite { name: "".to_string(), cases: Vec::new() };
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string => suite.name = inner.as_str().trim_matches('"').to_string(),
            Rule::test_case => suite.cases.push(parse_test_case(inner)),
            _ => {}
        }
    }
    suite
}

fn parse_test_case(pair: pest::iterators::Pair<Rule>) -> TestCase {
    let mut case = TestCase { name: "".to_string(), assertions: Vec::new() };
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string => case.name = inner.as_str().trim_matches('"').to_string(),
            Rule::assertion => case.assertions.push(parse_assertion(inner)),
            _ => {}
        }
    }
    case
}

fn parse_assertion(pair: pest::iterators::Pair<Rule>) -> Assertion {
    let mut target = "";
    let mut method = "";
    let mut expected = "";
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                if target.is_empty() { target = inner.as_str(); }
                else { method = inner.as_str(); }
            }
            Rule::string | Rule::number => expected = inner.as_str(),
            _ => {}
        }
    }
    Assertion { target: target.to_string(), method: method.to_string(), expected: expected.to_string() }
}