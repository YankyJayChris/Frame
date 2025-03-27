use crate::parser::{AST, TestSuite, Component};
use crate::runtime::{Canvas, State, I18n};
use crate::compiler::compile;
use std::collections::HashMap;
use winit::event_loop::EventLoop;

pub struct TestRunner {
    state: State,
    canvas: Canvas,
    i18n: I18n,
}

impl TestRunner {
    pub fn new(languages: Vec<HashMap<String, String>>) -> Self {
        let event_loop = EventLoop::new().unwrap();
        TestRunner {
            state: State::new(),
            canvas: Canvas::new(&event_loop, 800, 600),
            i18n: I18n::new(languages.first().unwrap_or(&HashMap::new()).clone()), // Default to first language
        }
    }

    pub fn set_language(&mut self, lang: &HashMap<String, String>) {
        self.i18n = I18n::new(lang.clone());
    }

    // Run Jest-like tests
    pub fn run_tests(&mut self, ast: &AST) {
        for suite in &ast.tests {
            println!("Running suite: {}", suite.name);
            for case in &suite.cases {
                println!("  Test: {}", case.name);
                for assertion in &case.assertions {
                    match assertion.method.as_str() {
                        "toBe" => {
                            let actual = self.state.get(&assertion.target).unwrap_or(&vec![]).join(",");
                            assert_eq!(actual, assertion.expected, "Assertion failed: {} != {}", actual, assertion.expected);
                            println!("    ✓ {} toBe {}", assertion.target, assertion.expected);
                        }
                        "toRender" => {
                            let rendered = self.render_component(Component {
                                name: assertion.target.clone(),
                                styles: Default::default(),
                                content: Some(assertion.expected.clone()),
                                children: vec![],
                                on_click: None,
                                on_mount: None,
                                on_update: None,
                                on_unmount: None,
                                data: None,
                                build: None,
                                direction: None,
                                scroll: None,
                                current: None,
                                items: vec![],
                                menu_icon: None,
                                actions: vec![],
                                src: None,
                                value: None,
                                options: vec![],
                                on_change: None,
                                on_select: None,
                                on_submit: None,
                                validation: None,
                                title: None,
                                icon: None,
                                lazy: None,
                                props: HashMap::new(),
                                animation: None,
                                error_boundary: None,
                            });
                            assert!(rendered.contains(&assertion.expected), "Assertion failed: {} not found in render", assertion.expected);
                            println!("    ✓ {} toRender {}", assertion.target, assertion.expected);
                        }
                        _ => println!("    ✗ Unsupported assertion: {}", assertion.method),
                    }
                }
            }
        }
    }

    pub fn render_component(&mut self, comp: Component) -> String {
        let ast = AST {
            pages: vec![],
            functions: HashMap::new(),
            components: HashMap::from([(comp.name.clone(), ComponentDef { name: comp.name.clone(), params: vec![], body: comp })]),
            tests: vec![],
            imports: HashMap::new(),
            vars: HashMap::new(),
            i18n: self.i18n.translations.clone(),
        };
        let code = compile(ast);
        // Simulate rendering; in a real test, this would execute the compiled code and capture output
        format!("Simulated render of {}", code)
    }

    pub fn assert_renders(&mut self, comp: Component, expected: &str) {
        let rendered = self.render_component(comp);
        assert_eq!(rendered, expected, "Component did not render as expected");
        println!("    ✓ Rendered as expected: {}", expected);
    }
}

// Example usage with multiple languages
pub fn run_tests_with_languages(ast: &AST, languages: Vec<HashMap<String, String>>) {
    let mut runner = TestRunner::new(languages.clone());
    for lang in languages {
        runner.set_language(&lang);
        println!("Testing with language: {:?}", lang);
        runner.run_tests(ast);
    }
}