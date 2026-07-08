//! `frame start` — scaffold a new Frame project.
//!
//! Supports two architectures:
//! - **Clean Architecture**: domain/usecases/data/presentation layers
//! - **MVC**: models/views/controllers

use std::fs;
use std::path::Path;

/// Architecture choice for the new project.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Architecture {
    CleanArchitecture,
    Mvc,
}

/// Scaffold a new Frame project at `{cwd}/{name}`.
pub fn scaffold_project(name: &str, arch: Architecture) -> std::io::Result<()> {
    let root = Path::new(name);
    if root.exists() {
        eprintln!("✗ Directory '{}' already exists.", name);
        std::process::exit(1);
    }

    println!("Creating Frame project: {}", name);

    // Common directories
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("assets/fonts"))?;
    fs::create_dir_all(root.join("assets/images"))?;
    fs::create_dir_all(root.join("frame_modules"))?;

    match arch {
        Architecture::CleanArchitecture => scaffold_clean(root)?,
        Architecture::Mvc               => scaffold_mvc(root)?,
    }

    // Common files
    write_project_fr(root, name, arch)?;
    write_frame_config(root, name)?;
    write_gitignore(root)?;
    write_readme(root, name, arch)?;

    // Tests directory + sample test file (always created regardless of arch)
    write_sample_tests(root, name, arch)?;

    println!("✓ Created '{}'", name);
    println!();
    println!("  Get started:");
    println!("    cd {}", name);
    println!("    frame check");
    println!("    frame build");
    println!("    frame test          # run sample tests");
    println!("    frame deploy android");
    println!("    frame deploy ios");
    Ok(())
}

// ─── Clean Architecture scaffold ──────────────────────────────────────────────

fn scaffold_clean(root: &Path) -> std::io::Result<()> {
    // Domain layer
    fs::create_dir_all(root.join("src/domain/entities"))?;
    fs::create_dir_all(root.join("src/domain/usecases"))?;
    fs::create_dir_all(root.join("src/domain/repositories"))?;
    // Data layer
    fs::create_dir_all(root.join("src/data/repositories"))?;
    fs::create_dir_all(root.join("src/data/models"))?;
    // Presentation layer
    fs::create_dir_all(root.join("src/presentation/pages"))?;
    fs::create_dir_all(root.join("src/presentation/components"))?;
    fs::create_dir_all(root.join("src/presentation/state"))?;

    // Sample entity
    fs::write(root.join("src/domain/entities/User.fr"),
        "// User entity\nconst User = \"{ id: string, name: string, email: string }\"\n")?;

    // Sample use case
    fs::write(root.join("src/domain/usecases/GetUser.fr"),
        "// GetUser use case\nfn getUser: async (id: string) => {\n    result = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n    return result\n}\n")?;

    // Sample repository interface
    fs::write(root.join("src/domain/repositories/UserRepository.fr"),
        "// UserRepository interface\n// Implement in data/repositories/\n")?;

    // Sample data model
    fs::write(root.join("src/data/models/UserModel.fr"),
        "// UserModel — maps API response to User entity\n:store UserStore {\n    user: object = null\n    is_loading: bool = false\n\n    fn load: async (id: string) => {\n        UserStore.is_loading = true\n        UserStore.user = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n        UserStore.is_loading = false\n    }\n}\n")?;

    // Sample page
    fs::write(root.join("src/presentation/pages/HomePage.fr"),
        "import { text, button, column, scaffold } \"frame-core\"\n\npage: {\n    name: \"Home\"\n    route: \"/\"\n    children: [\n        scaffold: {\n            children: [\n                column: {\n                    styles: { padding: 16dp }\n                    children: [\n                        text: { content: \"Welcome to Frame!\" }\n                        button: {\n                            content: \"Get Started\"\n                            on_click: navigate(\"/profile\")\n                        }\n                    ]\n                }\n            ]\n        }\n    ]\n}\n")?;

    // Sample component
    fs::write(root.join("src/presentation/components/UserCard.fr"),
        "component UserCard: {\n    props: {\n        name: string\n        email: string\n    }\n    children: [\n        card: {\n            styles: { padding: 16dp margin: 8dp }\n            children: [\n                text: { content: $name }\n                text: { content: $email }\n            ]\n        }\n    ]\n}\n")?;

    // Sample store (presentation state)
    fs::write(root.join("src/presentation/state/AppStore.fr"),
        ":store AppStore {\n    theme: string = \"light\"\n    is_authenticated: bool = false\n    auth_token: string = \"\"\n\n    persist: {\n        auth_token: secure\n        theme: local\n    }\n\n    fn setTheme: (t: string) => {\n        AppStore.theme = t\n    }\n\n    fn logout: () => {\n        AppStore.is_authenticated = false\n        AppStore.auth_token = \"\"\n    }\n}\n")?;

    Ok(())
}

// ─── MVC scaffold ─────────────────────────────────────────────────────────────

fn scaffold_mvc(root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/models"))?;
    fs::create_dir_all(root.join("src/views/pages"))?;
    fs::create_dir_all(root.join("src/views/components"))?;
    fs::create_dir_all(root.join("src/controllers"))?;

    // Sample model
    fs::write(root.join("src/models/User.fr"),
        ":store UserStore {\n    user: object = null\n    is_loading: bool = false\n    auth_token: string = \"\"\n\n    persist: { auth_token: secure }\n\n    fn load: async (id: string) => {\n        UserStore.is_loading = true\n        UserStore.user = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n        UserStore.is_loading = false\n    }\n}\n")?;

    // Sample view page
    fs::write(root.join("src/views/pages/HomePage.fr"),
        "import { text, button, column, scaffold } \"frame-core\"\n\npage: {\n    name: \"Home\"\n    route: \"/\"\n    children: [\n        scaffold: {\n            children: [\n                column: {\n                    styles: { padding: 16dp }\n                    children: [\n                        text: { content: \"Welcome to Frame!\" }\n                        button: {\n                            content: \"Get Started\"\n                            on_click: navigate(\"/profile\")\n                        }\n                    ]\n                }\n            ]\n        }\n    ]\n}\n")?;

    // Sample component
    fs::write(root.join("src/views/components/UserCard.fr"),
        "component UserCard: {\n    props: { name: string  email: string }\n    children: [\n        card: {\n            styles: { padding: 16dp }\n            children: [\n                text: { content: $name }\n                text: { content: $email }\n            ]\n        }\n    ]\n}\n")?;

    // Sample controller
    fs::write(root.join("src/controllers/UserController.fr"),
        "fn loadUser: async (id: string) => {\n    result = wait:fetch(\"/api/users/$id\", { method: \"GET\" })\n    return result\n}\n\nfn saveUser: async (user: object) => {\n    wait:fetch(\"/api/users\", { method: \"POST\"  body: { user: user } })\n}\n")?;

    Ok(())
}

// ─── Common generated files ───────────────────────────────────────────────────

fn write_project_fr(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let page_import = match arch {
        Architecture::CleanArchitecture => "./presentation/pages/HomePage.fr",
        Architecture::Mvc               => "./views/pages/HomePage.fr",
    };
    let content = format!(
        ":vars {{\n    $primary: \"#007BFF\"\n    $secondary: \"#6C757D\"\n    $spacing: 16dp\n    $radius: 8dp\n}}\n\n\
         :i18n {{\n    app_title: \"{name}\"\n    welcome: \"Welcome\"\n}}\n\n\
         :breakpoints {{\n    sm: 360dp\n    md: 600dp\n    lg: 900dp\n    xl: 1200dp\n}}\n\n\
         :typography {{\n    headline: {{ font_size: 24sp  font_weight: \"bold\" }}\n    body: {{ font_size: 16sp }}\n    caption: {{ font_size: 12sp  color: \"$secondary\" }}\n}}\n\n\
         import {{ text, button, column, row, scaffold }} \"frame-core\"\n\
         import {{ HomePage }} \"{page_import}\"\n\n\
         page: {{\n    name: \"Splash\"\n    route: \"/\"\n    before_enter: \"checkAuth\"\n    styles: {{\n        background: $primary\n        width: 100%\n        height: 100%\n    }}\n    children: [\n        column: {{\n            styles: {{ align: center  justify: center  width: 100%  height: 100% }}\n            children: [\n                text: {{\n                    content: t:\"app_title\"\n                    styles: {{ color: \"#FFFFFF\"  font_size: 32sp  font_weight: \"bold\" }}\n                }}\n            ]\n        }}\n    ]\n}}\n\nfn checkAuth: async () => {{\n    // Navigate to home after splash\n}}\n"
    );
    fs::write(root.join("src/project.fr"), content)
}

fn write_frame_config(root: &Path, name: &str) -> std::io::Result<()> {
    let safe = name.to_lowercase().replace(' ', "_");
    let content = format!(
        "{{\n  \"name\": \"{name}\",\n  \"bundle_id\": \"com.example.{safe}\",\n  \"version\": \"1.0.0\",\n  \"build_number\": \"1\",\n  \"render_mode\": \"native\",\n  \"min_android_sdk\": 24,\n  \"min_ios\": \"16.0\",\n  \"plugins\": []\n}}\n"
    );
    fs::write(root.join("frame.config.json"), content)
}

fn write_gitignore(root: &Path) -> std::io::Result<()> {
    fs::write(root.join(".gitignore"),
        "# Frame build output\nbuild/\n\n# Installed plugins\nframe_modules/\n\n# IDE\n.vscode/\n.idea/\n*.DS_Store\n*.swp\n")
}

fn write_sample_tests(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    fs::create_dir_all(root.join("src/tests"))?;

    // ── Store/logic test (architecture-aware paths) ───────────────────────────
    let store_name = match arch {
        Architecture::CleanArchitecture => "AppStore",
        Architecture::Mvc               => "UserStore",
    };

    let store_test = format!(
        r#"// {store_name} — unit tests
// Run with: frame test

describe: "{store_name}" => {{

  it: "has correct initial state" => {{
    expect: {store_name}.theme   .toBe: "light"
    expect: {store_name}.is_authenticated .toBeFalse:
  }}

  it: "setTheme updates the theme field" => {{
    {store_name}.setTheme("dark")
    expect: {store_name}.theme .toBe: "dark"
  }}

  it: "logout clears auth state" => {{
    // Arrange: simulate a logged-in state
    {store_name}.is_authenticated = true
    {store_name}.auth_token = "abc123"

    // Act
    {store_name}.logout()

    // Assert
    expect: {store_name}.is_authenticated .toBeFalse:
    expect: {store_name}.auth_token       .toBe: ""
  }}

}}
"#,
        store_name = store_name,
    );
    fs::write(root.join(format!("src/tests/{store_name}.test.fr")), &store_test)?;

    // ── HTTP / fetch test with mock: ──────────────────────────────────────────
    let fetch_test = format!(
        r#"// API integration tests — uses mock: to intercept wait:fetch calls
// Run with: frame test

describe: "API" => {{

  it: "fetches a user successfully" => {{
    mock: {{
      url: "/api/users/1"
      response: {{ id: "1"  name: "Jane Smith"  email: "jane@example.com" }}
    }}

    result = wait:fetch("/api/users/1", {{ method: "GET" }})

    expect: result.name  .toBe: "Jane Smith"
    expect: result.email .toBe: "jane@example.com"
  }}

  it: "handles fetch error gracefully" => {{
    mock: {{
      url: "/api/users/999"
      response: {{ error: "Not found" }}
      status: 404
    }}

    result = wait:fetch("/api/users/999", {{ method: "GET" }})

    expect: result.error .toBe: "Not found"
  }}

}}
"#
    );
    fs::write(root.join("src/tests/api.test.fr"), &fetch_test)?;

    // ── Navigation / page test ────────────────────────────────────────────────
    let nav_test = format!(
        r#"// Navigation tests — verify route transitions
// Run with: frame test

describe: "Navigation" => {{

  it: "navigates to home route" => {{
    navigate("/")
    expect: current_route .toBe: "/"
  }}

  it: "navigate_back returns to previous route" => {{
    navigate("/")
    navigate("/profile")
    navigate_back()
    expect: current_route .toBe: "/"
  }}

}}
"#
    );
    fs::write(root.join("src/tests/navigation.test.fr"), &nav_test)?;

    Ok(())
}

fn write_readme(root: &Path, name: &str, arch: Architecture) -> std::io::Result<()> {
    let arch_name = match arch {
        Architecture::CleanArchitecture => "Clean Architecture",
        Architecture::Mvc               => "MVC",
    };
    let arch_desc = match arch {
        Architecture::CleanArchitecture =>
            "```\nsrc/\n  domain/         # Entities, use cases, repository interfaces\n  data/           # Repository implementations, API models\n  presentation/   # Pages, components, stores\n```",
        Architecture::Mvc =>
            "```\nsrc/\n  models/         # Data stores and API models\n  views/          # Pages and components\n  controllers/    # Business logic functions\n```",
    };
    let content = format!(
        "# {name}\n\nA Frame cross-platform mobile app using **{arch_name}**.\n\n## Project Structure\n\n{arch_desc}\n\n## Commands\n\n```bash\n# Check your environment\nframe check\n\n# Build the project\nframe build\n\n# Deploy to Android\nframe deploy android\n\n# Deploy to iOS  \nframe deploy ios\n\n# Run tests\nframe test\n\n# Start hot-reload preview\nframe preview\n```\n\nWrite all your app code in `.fr` files and `frame.config.json` — that's it.\nFrame handles all Android and iOS code generation for you.\n"
    );
    fs::write(root.join("README.md"), content)
}
