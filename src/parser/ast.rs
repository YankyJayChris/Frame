//! AST node type definitions for the Frame language.
//!
//! All types are self-contained here. `parser/mod.rs` re-exports from this
//! module and keeps only the `parse_project` stub.

use std::collections::HashMap;

// ─── String interpolation ─────────────────────────────────────────────────────

/// A segment of a string interpolation expression.
#[derive(Debug, Clone)]
pub enum InterpolatedSegment {
    /// A plain string literal part.
    Literal(String),
    /// An embedded expression part.
    Expr(Box<Expr>),
}

// ─── Primitive / enum types ──────────────────────────────────────────────────

/// The Frame type system: mirrors the type table in design §2.3.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FRType {
    #[default]
    String_,
    Int,
    Float,
    Bool,
    Object,
    List,
    Nullable(Box<FRType>),
    /// A user-defined type: `:enum`, `:type`, or `:obj` name.
    Custom(String),
}

/// Overflow behaviour for layout containers.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum OverflowValue {
    #[default]
    Visible,
    Hidden,
    Scroll,
    ScrollX,
    ScrollY,
    Auto,
}

/// How overflowing text is truncated.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TextOverflowValue {
    #[default]
    Clip,
    Ellipsis,
    Fade,
}

/// How an image fills its container.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ImageFitValue {
    Cover,
    #[default]
    Contain,
    Fill,
    None_,
    ScaleDown,
}

/// How child content is clipped at container edges.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ClipBehavior {
    None_,
    Hard,
    #[default]
    AntiAliased,
}

/// Scroll-snap alignment for scroll containers.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ScrollSnap {
    #[default]
    None_,
    Start,
    Center,
    End,
}

/// Default child alignment for stack: containers.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum StackAlignment {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Absolute position overrides for a child inside stack:.
#[derive(Debug, Clone, Default)]
pub struct PositionedProps {
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
}

/// Easing curves for animations.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum EasingType {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Spring,
}

/// Where a persisted store field is stored on-device.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PersistStrategy {
    #[default]
    Local,
    Secure,
}

// ─── Styles ──────────────────────────────────────────────────────────────────

/// Fully-typed style block — replaces the old `HashMap<String, String>` stub.
#[derive(Debug, Clone, Default)]
pub struct Styles {
    // Layout
    pub width: Option<String>,
    pub height: Option<String>,
    pub min_width: Option<String>,
    pub max_width: Option<String>,
    pub min_height: Option<String>,
    pub max_height: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub flex: Option<String>,
    pub flex_wrap: Option<String>,
    pub direction: Option<String>,
    pub align: Option<String>,
    pub justify: Option<String>,
    pub gap: Option<String>,
    pub aspect_ratio: Option<String>,
    // Spacing
    pub margin: Option<String>,
    pub margin_top: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub margin_right: Option<String>,
    pub padding: Option<String>,
    pub padding_top: Option<String>,
    pub padding_bottom: Option<String>,
    pub padding_left: Option<String>,
    pub padding_right: Option<String>,
    // Appearance
    pub background: Option<String>,
    pub color: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub border: Option<String>,
    pub border_radius: Option<String>,
    pub opacity: Option<String>,
    pub visible: Option<bool>,
    pub safe_area: Option<bool>, // default true — extends content into safe area
    // Overflow
    pub overflow: OverflowValue,
    pub overflow_x: Option<OverflowValue>,
    pub overflow_y: Option<OverflowValue>,
    pub clip_behavior: ClipBehavior,
    // Text overflow
    pub text_overflow: TextOverflowValue,
    pub max_lines: Option<u32>,
    pub line_clamp: Option<u32>, // alias for max_lines
    // Image
    pub image_fit: ImageFitValue,
    // Scroll config
    pub scroll_indicator: Option<bool>,
    pub scroll_snap: ScrollSnap,
    pub scroll_enabled: Option<String>, // supports $state.field binding
    pub on_scroll: Option<String>,
    pub on_scroll_end: Option<String>,
    // Breakpoint overrides: breakpoint_name → Styles
    pub breakpoint_overrides: HashMap<String, Box<Styles>>,
    // Forward-compat: unrecognised/extra props
    pub extra: HashMap<String, String>,
}

// ─── Animation ───────────────────────────────────────────────────────────────

/// A single property animation descriptor.
#[derive(Debug, Clone, Default)]
pub struct Animation {
    pub property: String,
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub from: String,
    pub to: String,
    pub easing: EasingType,
    pub repeat: bool,
    pub auto_reverse: bool,
}

// ─── Object type declaration ──────────────────────────────────────────────────

/// A `:obj TypeName { field: type ... }` declaration — a named data model.
/// Used for domain entities and API response shapes.
/// Compiles to a Kotlin data class and a Swift struct.
#[derive(Debug, Clone, Default)]
pub struct ObjDef {
    pub name: String,
    pub fields: Vec<ObjField>,
}

/// A single typed field in an :obj declaration.
#[derive(Debug, Clone, Default)]
pub struct ObjField {
    pub name: String,
    pub type_: FRType,
    pub optional: bool,
}

// ─── Store ───────────────────────────────────────────────────────────────────

/// One `:store` slice — state fields, actions, and persistence strategy.
#[derive(Debug, Clone, Default)]
pub struct StoreSlice {
    pub name: String,
    pub fields: HashMap<String, StoreField>,
    pub actions: HashMap<String, Function>,
    pub persist: HashMap<String, PersistStrategy>,
}

/// A typed field within a store slice.
#[derive(Debug, Clone, Default)]
pub struct StoreField {
    pub name: String,
    pub type_: FRType,
    pub default: Option<Expr>,
}

// ─── Function / Stmt / Expr ──────────────────────────────────────────────────

/// A named (optionally async) function definition.
#[derive(Debug, Clone, Default)]
pub struct Function {
    pub name: String,
    pub is_async: bool,
    /// Each param: `(name, type, optional_default_expr)` — plan §1e
    pub params: Vec<(String, FRType, Option<Expr>)>,
    pub return_type: Option<FRType>,
    pub body: Vec<Stmt>,
}

/// A typed variable declaration.
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub mutable: bool,
    pub type_: Option<FRType>,
    pub initializer: Option<Expr>,
}

/// A statement in a function body.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Typed variable declaration: `:var x: type = expr`
    VarDecl(VarDecl),
    /// Variable assignment: `x = expr`
    Assign(String, Expr),
    /// `if expr { ... } else { ... }`
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    /// `for item in expr { ... }`
    For(String, Expr, Vec<Stmt>),
    /// `switch expr { case: [...] }`
    Switch(Expr, Vec<(Expr, Vec<Stmt>)>),
    /// Synchronous function call statement.
    Call(CallExpr),
    /// `wait:name(args)` — async user-function call.
    Wait(CallExpr),
    /// `wait:fetch(url, opts)` — async HTTP fetch.
    WaitFetch(FetchExpr),
    /// `return expr`
    Return(Expr),
    /// `try { ... } catch (err) { ... } finally { ... }`
    TryCatch {
        body: Vec<Stmt>,
        catch_param: String,
        catch_body: Vec<Stmt>,
        finally_body: Option<Vec<Stmt>>,
    },
    /// `plugin: { name: "..." method: "..." params: { ... } }` — native bridge call.
    PluginCall(PluginCall),
}

/// An expression in the AST.
/// Options passed to navigate() — all optional, all have sensible defaults.
#[derive(Debug, Clone, Default)]
pub struct NavOptions {
    /// Replace current entry in back stack instead of pushing.
    pub replace: bool,
    /// Pop the entire back stack to root before navigating.
    pub clear_stack: bool,
    /// Prevent navigating to the same route twice (Android: launchSingleTop).
    pub single_top: bool,
    /// Named transition animation, e.g. "slide_up", "fade", "slide_left".
    pub transition: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    /// A literal value.
    Literal(Value),
    /// A variable reference, e.g. `$name` or `name`.
    Var(String),
    /// A local state field reference, e.g. `state.count`.
    StateField(String),
    /// A store field reference, e.g. `AuthStore.token`.
    StoreField(String, String),
    /// A binary operation, e.g. `a + b`.
    BinOp(Box<Expr>, Op, Box<Expr>),
    /// A function call expression.
    Call(CallExpr),
    /// Null coalescing: `expr ?? fallback`.
    NullCoalesce(Box<Expr>, Box<Expr>),
    /// Safe navigation chain: `a?.b?.c`.
    SafeNav(Vec<String>),
    /// Method call: `receiver.method(args)`.
    MethodCall(Box<Expr>, String, Vec<Expr>),
    /// An inline lambda: `(params) => { body }`.
    Lambda(Vec<String>, Vec<Stmt>),
    /// A string with embedded expressions: `"Hello \(name)!"` or `"Hello ${name}!"`
    Interpolated(Vec<InterpolatedSegment>),
    /// `navigate("/route")` or `navigate("/route", replace: true, transition: "slide_up")`
    Navigate(Box<Expr>, NavOptions),
    /// `navigate_replace("/route")` — replace current back-stack entry.
    NavigateReplace(Box<Expr>),
    /// `navigate_back()` — pop one entry.
    NavigateBack,
    /// `navigate_back_to("/home")` — pop to a specific route.
    NavigateBackTo(Box<Expr>),
    /// `navigate_modal("/sheet")` — present modally.
    NavigateModal(Box<Expr>),
    /// `navigate_dismiss()` — dismiss current modal.
    NavigateDismiss,
}

/// Default Expr is a Null literal.
impl Default for Expr {
    fn default() -> Self {
        Expr::Literal(Value::Null)
    }
}

/// A literal value that can appear in expressions.
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

/// Binary / comparison / logical operators.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
}

/// A synchronous or `wait:`-prefixed function call.
#[derive(Debug, Clone, Default)]
pub struct CallExpr {
    pub func: String,
    pub args: Vec<Expr>,
    /// Named arguments: `name: value` pairs (plan §1g)
    pub named_args: Vec<(String, Expr)>,
}

/// A `wait:fetch(...)` HTTP request with then/catch branches.
#[derive(Debug, Clone, Default)]
pub struct FetchExpr {
    pub url: Expr,
    pub method: String,
    pub headers: HashMap<String, Expr>,
    pub body: Option<Expr>,
    pub timeout_ms: Option<u32>,
    pub then_branch: Vec<Stmt>,
    pub catch_branch: Vec<Stmt>,
}

// ─── Component / Page / ComponentDef ─────────────────────────────────────────

/// A prop declaration in a `component` definition.
#[derive(Debug, Clone, Default)]
pub struct PropDef {
    pub name: String,
    pub type_: FRType,
    pub required: bool,
    pub default: Option<Expr>,
}

/// A local-state field inside a `state:` block.
#[derive(Debug, Clone, Default)]
pub struct StateField {
    pub name: String,
    pub type_: FRType,
    pub default: Option<Expr>,
}

/// The full set of lifecycle and interaction event handlers for a node.
#[derive(Debug, Clone, Default)]
pub struct EventMap {
    pub on_click: Option<Expr>,
    pub on_change: Option<Expr>,
    pub on_submit: Option<Expr>,
    pub on_select: Option<Expr>,
    pub on_touch_start: Option<Expr>,
    pub on_touch_move: Option<Expr>,
    pub on_touch_end: Option<Expr>,
    /// Fires once after the node is first rendered/mounted.
    pub on_mount: Option<Expr>,
    /// Fires when `watch` dependencies change. Maps to `LaunchedEffect(key)` / `viewDidLayoutSubviews`.
    pub on_update: Option<Expr>,
    /// Optional dependency expression for `on_update` — e.g. a store field.
    /// Maps to `LaunchedEffect(watch_key)` in Compose.
    pub watch: Option<Expr>,
    /// Fires when the node is removed from the tree / view hierarchy.
    pub on_unmount: Option<Expr>,
}

/// A single node in the component tree (replaces the old `Component` stub).
#[derive(Debug, Clone, Default)]
pub struct ComponentNode {
    /// Built-in element kind or user-defined component name, e.g. `"text"`, `"Card"`.
    pub kind: String,
    pub props: HashMap<String, Expr>,
    pub styles: Styles,
    pub children: Vec<ComponentNode>,
    pub events: EventMap,
    pub animate: Vec<Animation>,
    /// `show_if: condition`
    pub show_if: Option<Expr>,
    /// For list nodes: `data: prop`
    pub data: Option<Expr>,
    /// For list nodes: `build: (item) => { ... }`
    pub build: Option<Function>,
    /// For stack: — default alignment of children that have no positioned: override
    pub alignment: StackAlignment,
    /// For children inside stack: — absolute position overrides
    pub positioned: Option<PositionedProps>,
    /// Attached `plugin:` call for this node (e.g. on a map_view or custom plugin node).
    pub plugin_call: Option<PluginCall>,
}

/// A reusable `component Name:` definition.
#[derive(Debug, Clone, Default)]
pub struct ComponentDef {
    pub name: String,
    pub props: HashMap<String, PropDef>,
    pub state: HashMap<String, StateField>,
    pub styles: Styles,
    pub children: Vec<ComponentNode>,
    pub events: EventMap,
    pub animate: Vec<Animation>,
    pub functions: HashMap<String, Function>,
}

/// A `page:` definition — a named, routed screen.
#[derive(Debug, Clone, Default)]
pub struct Page {
    pub name: String,
    pub route: String,
    /// Typed route params declared on the page, e.g. `params: { userId: string }`.
    /// At codegen time these become typed function/init parameters.
    pub params: Vec<(String, FRType)>,
    /// Called (as expr) before the page transition completes — use for auth guards.
    /// Maps to `LaunchedEffect(Unit)` / `viewWillAppear`.
    pub before_enter: Option<Expr>,
    /// Called (as expr) as the page begins to leave — use for cleanup.
    /// Maps to `DisposableEffect/onDispose` / `viewDidDisappear`.
    pub before_leave: Option<Expr>,
    /// Called after the page is fully visible.
    /// Maps to a second `LaunchedEffect(Unit)` / `viewDidAppear`.
    pub on_mount: Option<Expr>,
    /// Called when the page goes to background (app-level).
    /// Maps to `Lifecycle.Event.ON_PAUSE` observer / `sceneWillResignActive`.
    pub on_background: Option<Expr>,
    /// Called when the page returns to foreground (app-level).
    /// Maps to `Lifecycle.Event.ON_RESUME` observer / `sceneDidBecomeActive`.
    pub on_foreground: Option<Expr>,
    /// Called just before the page is fully removed.
    /// Maps to `DisposableEffect/onDispose` (combined with before_leave) / `viewDidDisappear`.
    pub on_unmount: Option<Expr>,
    pub styles: Styles,
    pub state: HashMap<String, StateField>,
    pub children: Vec<ComponentNode>,
}

// ─── Import / Const ───────────────────────────────────────────────────────────

/// An `import { X as Y } "path"` statement.
#[derive(Debug, Clone, Default)]
pub struct Import {
    /// Each entry is `(original_name, optional_alias)`.
    pub names: Vec<(String, Option<String>)>,
    pub path: String,
}

/// A compile-time constant value (`const name = value`).
#[derive(Debug, Clone)]
pub enum ConstValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Default for ConstValue {
    fn default() -> Self {
        ConstValue::Str(String::new())
    }
}

// ─── Breakpoints / Typography / ScreenContext ─────────────────────────────────

/// A named responsive breakpoint definition.
#[derive(Debug, Clone, Default)]
pub struct Breakpoint {
    pub name: String,
    pub min_width_dp: f32,
}

/// Runtime screen context passed to the responsive engine.
#[derive(Debug, Clone, Default)]
pub struct ScreenContext {
    pub width_dp: f32,
    pub height_dp: f32,
    pub breakpoint: String,
    pub is_phone: bool,
    pub is_tablet: bool,
    pub is_large: bool,
    /// `"portrait"` or `"landscape"`
    pub orientation: String,
}

/// A named typography scale entry (e.g. `headline`, `body`, `caption`).
#[derive(Debug, Clone, Default)]
pub struct TypographyStyle {
    pub name: String,
    pub font_size: String,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub line_height: Option<String>,
    pub letter_spacing: Option<String>,
    pub color: Option<String>,
    pub breakpoint_overrides: HashMap<String, Box<TypographyStyle>>,
}

// ─── Test types ───────────────────────────────────────────────────────────────

/// Mock HTTP configuration for a test case.
#[derive(Debug, Clone, Default)]
pub struct MockConfig {
    pub url_pattern: String,
    pub response: Value,
    pub status_code: u16,
}

/// A `describe:` test suite.
#[derive(Debug, Clone, Default)]
pub struct TestSuite {
    pub name: String,
    pub cases: Vec<TestCase>,
}

/// A single `it:` / `test:` case within a suite.
#[derive(Debug, Clone, Default)]
pub struct TestCase {
    pub name: String,
    pub mocks: Vec<MockConfig>,
    pub body: Vec<Stmt>,
    pub assertions: Vec<Assertion>,
}

/// A single `expect(...).to_be(...)` assertion.
#[derive(Debug, Clone, Default)]
pub struct Assertion {
    pub expr: Expr,
    pub matcher: Matcher,
    pub expected: Option<Expr>,
}

/// The matcher verb for an assertion.
#[derive(Debug, Clone, Default)]
pub enum Matcher {
    #[default]
    ToBe,
    ToEqual,
    ToContain,
    ToBeNull,
    ToBeTrue,
    ToBeFalse,
    ToThrow,
}

// ─── Plugin call ─────────────────────────────────────────────────────────────

/// A `plugin: { name: "..." method: "..." params: { ... } }` bridge call.
#[derive(Debug, Clone, Default)]
pub struct PluginCall {
    /// The plugin package name, e.g. `"frame_maps"`.
    pub plugin_name: String,
    /// The method to invoke on the native bridge, e.g. `"showMap"`.
    pub method: String,
    /// Named parameters passed to the native method.
    pub params: HashMap<String, Expr>,
}

/// A single enum variant with an optional string value.
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<String>,
}

/// A user-defined `:enum` declaration.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

/// A user-defined `:type` alias.
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub alias: String,
    pub target: FRType,
}

/// A single validation rule for a field.
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub arg: Option<Expr>,
}

/// A field in a `:validation` block.
#[derive(Debug, Clone)]
pub struct ValidationField {
    pub field: String,
    pub rules: Vec<ValidationRule>,
}

/// A `:validation` schema definition.
#[derive(Debug, Clone)]
pub struct ValidationSchema {
    pub name: String,
    pub fields: Vec<ValidationField>,
}

// ─── Top-level AST ────────────────────────────────────────────────────────────

/// The root of a parsed Frame project.
#[derive(Debug, Clone, Default)]
pub struct AST {
    pub vars: HashMap<String, String>,
    pub i18n: HashMap<String, String>,
    pub enums: HashMap<String, EnumDef>,
    pub type_aliases: HashMap<String, TypeAlias>,
    pub stores: HashMap<String, StoreSlice>,
    pub objects: HashMap<String, ObjDef>,
    pub imports: Vec<Import>,
    pub consts: HashMap<String, ConstValue>,
    pub pages: Vec<Page>,
    pub components: HashMap<String, ComponentDef>,
    pub functions: HashMap<String, Function>,
    pub tests: Vec<TestSuite>,
    pub breakpoints: Vec<Breakpoint>,
    pub typography: HashMap<String, TypographyStyle>,
    /// `:validation` schema blocks (plan §5a)
    pub validations: HashMap<String, ValidationSchema>,
    /// App-level lifecycle hooks — called once per app session.
    /// `on_launch`  → Android `Application.onCreate` / iOS `didFinishLaunchingWithOptions`
    /// `on_foreground` → Android `ProcessLifecycleOwner ON_START` / iOS `sceneWillEnterForeground`
    /// `on_background` → Android `ProcessLifecycleOwner ON_STOP`  / iOS `sceneDidEnterBackground`
    pub on_launch: Option<String>,
    pub on_foreground: Option<String>,
    pub on_background: Option<String>,
}
