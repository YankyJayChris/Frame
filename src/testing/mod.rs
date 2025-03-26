use crate::parser::{AST, TestSuite};
use crate::runtime::{Canvas, State};

// Run Jest-like tests
pub fn run_tests(ast: &AST) {
    for suite in &ast.tests {
        println!("Running suite: {}", suite.name);
        for case in &suite.cases {
            println!("  Test: {}", case.name);
            let mut state = State::new();
            let mut canvas = Canvas::new(&winit::event_loop::EventLoop::new().unwrap(), 800, 600);
            for assertion in &case.assertions {
                match assertion.method.as_str() {
                    "toBe" => {
                        let actual = state.get(&assertion.target).unwrap_or(&vec![]).join(",");
                        assert_eq!(actual, assertion.expected, "Assertion failed: {} != {}", actual, assertion.expected);
                        println!("    ✓ {} toBe {}", assertion.target, assertion.expected);
                    }
                    _ => println!("    ✗ Unsupported assertion: {}", assertion.method),
                }
            }
        }
    }
}