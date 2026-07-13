//! Pretty printer — serialises an `&AST` back to canonical `.fr` source text.
//!
//! The output is designed to round-trip through the Frame parser.

use crate::parser::ast::*;

/// Serialise an AST back to canonical .fr source text.
pub fn print(ast: &AST) -> String {
    let mut out = String::new();

    // :vars
    if !ast.vars.is_empty() {
        let mut entries: Vec<_> = ast.vars.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        out.push_str(":vars {\n");
        for (k, v) in &entries {
            // vars values: if looks like a quoted string, emit as-is quoted
            let formatted = format_var_value(v);
            out.push_str(&format!("    {}: {};\n", k, formatted));
        }
        out.push_str("}\n\n");
    }

    // :i18n
    if !ast.i18n.is_empty() {
        let mut entries: Vec<_> = ast.i18n.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        out.push_str(":i18n {\n");
        for (k, v) in &entries {
            out.push_str(&format!("    {}: \"{}\";\n", k, v));
        }
        out.push_str("}\n\n");
    }

    // imports
    for imp in &ast.imports {
        out.push_str(&print_import(imp));
        out.push('\n');
    }
    if !ast.imports.is_empty() {
        out.push('\n');
    }

    // consts
    let mut consts: Vec<_> = ast.consts.iter().collect();
    consts.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in &consts {
        out.push_str(&format!("const {} = {}\n", k, print_const_value(v)));
    }
    if !ast.consts.is_empty() {
        out.push('\n');
    }

    // :breakpoints
    if !ast.breakpoints.is_empty() {
        out.push_str(":breakpoints {\n");
        for bp in &ast.breakpoints {
            out.push_str(&format!("    {}: {}dp\n", bp.name, bp.min_width_dp));
        }
        out.push_str("}\n\n");
    }

    // :typography
    if !ast.typography.is_empty() {
        out.push_str(":typography {\n");
        let mut typo: Vec<_> = ast.typography.iter().collect();
        typo.sort_by_key(|(k, _)| k.as_str());
        for (_, ts) in &typo {
            out.push_str(&print_typography_style(ts, 1));
        }
        out.push_str("}\n\n");
    }

    // :store
    let mut stores: Vec<_> = ast.stores.iter().collect();
    stores.sort_by_key(|(k, _)| k.as_str());
    for (_, store) in &stores {
        out.push_str(&print_store(store));
        out.push('\n');
    }

    // :obj declarations
    let mut objects: Vec<_> = ast.objects.iter().collect();
    objects.sort_by_key(|(k, _)| k.as_str());
    for (_, obj) in &objects {
        out.push_str(&print_obj(obj));
        out.push('\n');
    }

    // :app lifecycle block (only emitted when at least one hook is declared)
    if ast.default_route.is_some() || ast.on_launch.is_some()
        || ast.on_foreground.is_some() || ast.on_background.is_some()
    {
        out.push_str(":app {\n");
        if let Some(r) = &ast.default_route   { out.push_str(&format!("    default_route: \"{r}\"\n")); }
        if let Some(f) = &ast.on_launch       { out.push_str(&format!("    on_launch:     {f}\n")); }
        if let Some(f) = &ast.on_foreground   { out.push_str(&format!("    on_foreground: {f}\n")); }
        if let Some(f) = &ast.on_background   { out.push_str(&format!("    on_background: {f}\n")); }
        out.push_str("}\n\n");
    }

    // pages
    for page in &ast.pages {
        out.push_str(&print_page(page));
        out.push('\n');
    }

    // components
    let mut components: Vec<_> = ast.components.iter().collect();
    components.sort_by_key(|(k, _)| k.as_str());
    for (_, comp) in &components {
        out.push_str(&print_component_def(comp));
        out.push('\n');
    }

    // functions
    let mut functions: Vec<_> = ast.functions.iter().collect();
    functions.sort_by_key(|(k, _)| k.as_str());
    for (_, func) in &functions {
        out.push_str(&print_function(func, 0));
        out.push('\n');
    }

    // test suites
    for suite in &ast.tests {
        out.push_str(&print_test_suite(suite));
        out.push('\n');
    }

    out
}

// ─── Var value formatting ─────────────────────────────────────────────────────

fn format_var_value(v: &str) -> String {
    // If the value already looks like a dimension/number, emit as-is; otherwise quote it
    if looks_like_dimension(v) || looks_like_number(v) {
        v.to_string()
    } else {
        format!("\"{}\"", v)
    }
}

fn looks_like_dimension(s: &str) -> bool {
    let units = ["dp", "sp", "px", "%", "ms"];
    for u in &units {
        if s.ends_with(u) {
            let prefix = &s[..s.len() - u.len()];
            if prefix.parse::<f64>().is_ok() {
                return true;
            }
        }
    }
    false
}

fn looks_like_number(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

/// Returns a style value string properly formatted for .fr output.
/// Values that look like dimensions, numbers, booleans, or plain keywords
/// are emitted as-is. All other values (colors, strings, etc.) get quoted.
fn format_style_value(v: &str) -> String {
    // Already looks like a dimension (e.g. 100%, 10dp, 24sp)
    if looks_like_dimension(v) {
        return v.to_string();
    }
    // Plain number
    if looks_like_number(v) {
        return v.to_string();
    }
    // Boolean keywords
    if v == "true" || v == "false" || v == "null" {
        return v.to_string();
    }
    // Dollar variable
    if v.starts_with('$') {
        return v.to_string();
    }
    // Plain lowercase keyword (style enum values like hidden, scroll, etc.)
    // A keyword is all lowercase letters/underscores with no special chars
    if v.chars().all(|c| c.is_ascii_lowercase() || c == '_') && !v.is_empty() {
        return v.to_string();
    }
    // Everything else: wrap in double quotes
    format!("\"{}\"", v)
}

// ─── Const value ─────────────────────────────────────────────────────────────

fn print_const_value(v: &ConstValue) -> String {
    match v {
        ConstValue::Str(s)   => format!("\"{}\"", s),
        ConstValue::Int(n)   => n.to_string(),
        ConstValue::Float(f) => format_float(*f),
        ConstValue::Bool(b)  => b.to_string(),
    }
}

fn format_float(f: f64) -> String {
    if f.fract() == 0.0 {
        format!("{:.1}", f)
    } else {
        f.to_string()
    }
}

// ─── Import ───────────────────────────────────────────────────────────────────

fn print_import(imp: &Import) -> String {
    let names: Vec<String> = imp.names.iter().map(|(orig, alias)| {
        match alias {
            Some(a) => format!("{} as {}", orig, a),
            None    => orig.clone(),
        }
    }).collect();
    format!("import {{ {} }} \"{}\"", names.join(", "), imp.path)
}

// ─── Typography ───────────────────────────────────────────────────────────────

fn print_typography_style(ts: &TypographyStyle, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let inner_pad = "    ".repeat(indent + 1);
    let mut out = format!("{}{}: {{\n", pad, ts.name);
    out.push_str(&format!("{}font_size: {}\n", inner_pad, ts.font_size));
    if let Some(fw) = &ts.font_weight {
        out.push_str(&format!("{}font_weight: \"{}\"\n", inner_pad, fw));
    }
    if let Some(ff) = &ts.font_family {
        out.push_str(&format!("{}font_family: \"{}\"\n", inner_pad, ff));
    }
    if let Some(lh) = &ts.line_height {
        out.push_str(&format!("{}line_height: {}\n", inner_pad, lh));
    }
    if let Some(ls) = &ts.letter_spacing {
        out.push_str(&format!("{}letter_spacing: {}\n", inner_pad, ls));
    }
    if let Some(c) = &ts.color {
        out.push_str(&format!("{}color: \"{}\"\n", inner_pad, c));
    }
    // Breakpoint overrides inside typography
    let mut bp_overrides: Vec<_> = ts.breakpoint_overrides.iter().collect();
    bp_overrides.sort_by_key(|(k, _)| k.as_str());
    for (bp_name, bp_style) in &bp_overrides {
        out.push_str(&format!("{}@{} {{\n", inner_pad, bp_name));
        let deep_pad = "    ".repeat(indent + 2);
        out.push_str(&format!("{}font_size: {}\n", deep_pad, bp_style.font_size));
        if let Some(fw) = &bp_style.font_weight {
            out.push_str(&format!("{}font_weight: \"{}\"\n", deep_pad, fw));
        }
        if let Some(c) = &bp_style.color {
            out.push_str(&format!("{}color: \"{}\"\n", deep_pad, c));
        }
        out.push_str(&format!("{}}}\n", inner_pad));
    }
    out.push_str(&format!("{}}}\n", pad));
    out
}

// ─── Store ────────────────────────────────────────────────────────────────────

fn print_store(store: &StoreSlice) -> String {
    let mut out = format!(":store {} {{\n", store.name);

    // Fields (sorted for determinism)
    let mut fields: Vec<_> = store.fields.iter().collect();
    fields.sort_by_key(|(k, _)| k.as_str());
    for (_, field) in &fields {
        let default_str = match &field.default {
            Some(e) => format!(" = {}", print_expr(e)),
            None => String::new(),
        };
        out.push_str(&format!("    {}: {}{}\n", field.name, print_type(&field.type_), default_str));
    }

    // persist block
    if !store.persist.is_empty() {
        out.push_str("    persist: {\n");
        let mut persist: Vec<_> = store.persist.iter().collect();
        persist.sort_by_key(|(k, _)| k.as_str());
        for (field_name, strategy) in &persist {
            let s = match strategy {
                PersistStrategy::Secure => "secure",
                PersistStrategy::Local  => "local",
            };
            out.push_str(&format!("        {}: {}\n", field_name, s));
        }
        out.push_str("    }\n");
    }

    // Actions (sorted)
    let mut actions: Vec<_> = store.actions.iter().collect();
    actions.sort_by_key(|(k, _)| k.as_str());
    for (_, func) in &actions {
        out.push_str(&print_store_fn(func));
    }

    out.push('}');
    out
}

fn print_store_fn(func: &Function) -> String {
    let async_kw = if func.is_async { "async " } else { "" };
    let params = print_fn_params(&func.params);
    let mut out = format!("    fn {}: {}({}) => {{\n", func.name, async_kw, params);
    for stmt in &func.body {
        out.push_str(&print_stmt(stmt, 2));
    }
    out.push_str("    }\n");
    out
}

// ─── :obj ─────────────────────────────────────────────────────────────────────

fn print_obj(obj: &ObjDef) -> String {
    let mut out = format!(":obj {} {{\n", obj.name);
    for (i, field) in obj.fields.iter().enumerate() {
        let opt = if field.optional { "?" } else { "" };
        let comma = if i + 1 < obj.fields.len() { "," } else { "" };
        out.push_str(&format!("    {}: {}{}{}\n", field.name, print_type(&field.type_), opt, comma));
    }
    out.push('}');
    out
}

// ─── Function ─────────────────────────────────────────────────────────────────

fn print_function(func: &Function, base_indent: usize) -> String {
    let pad = "    ".repeat(base_indent);
    let async_kw = if func.is_async { "async " } else { "" };
    let params = print_fn_params(&func.params);
    let mut out = format!("{}fn {}: {}({}) => {{\n", pad, func.name, async_kw, params);
    for stmt in &func.body {
        out.push_str(&print_stmt(stmt, base_indent + 1));
    }
    out.push_str(&format!("{}}}", pad));
    out
}

fn print_fn_params(params: &[(String, FRType, Option<Expr>)]) -> String {
    params.iter().map(|(name, t, default)| {
        match default {
            Some(d) => format!("{}: {} = {}", name, print_type(t), print_expr(d)),
            None    => format!("{}: {}", name, print_type(t)),
        }
    }).collect::<Vec<_>>().join(", ")
}

fn print_type(t: &FRType) -> String {
    match t {
        FRType::String_       => "string".to_string(),
        FRType::Int           => "int".to_string(),
        FRType::Float         => "float".to_string(),
        FRType::Bool          => "bool".to_string(),
        FRType::Object        => "object".to_string(),
        FRType::List          => "list".to_string(),
        FRType::Nullable(inner) => format!("{}?", print_type(inner)),
        FRType::Custom(name) => name.clone(),
    }
}

// ─── Page ─────────────────────────────────────────────────────────────────────

fn print_page(page: &Page) -> String {
    let mut out = String::from("page: {\n");
    out.push_str(&format!("    name: \"{}\"\n", page.name));
    out.push_str(&format!("    route: \"{}\"\n", page.route));
    if !page.params.is_empty() {
        out.push_str("    params: {\n");
        for (name, type_) in &page.params {
            out.push_str(&format!("        {}: {}\n", name, print_type(type_)));
        }
        out.push_str("    }\n");
    }
    if let Some(be) = &page.before_enter {
        out.push_str(&format!("    before_enter: {}\n", print_expr(be)));
    }
    if let Some(bl) = &page.before_leave {
        out.push_str(&format!("    before_leave: {}\n", print_expr(bl)));
    }
    if let Some(om) = &page.on_mount {
        out.push_str(&format!("    on_mount: {}\n", print_expr(om)));
    }
    if let Some(ou) = &page.on_unmount {
        out.push_str(&format!("    on_unmount: {}\n", print_expr(ou)));
    }
    if let Some(of_) = &page.on_foreground {
        out.push_str(&format!("    on_foreground: {}\n", print_expr(of_)));
    }
    if let Some(ob) = &page.on_background {
        out.push_str(&format!("    on_background: {}\n", print_expr(ob)));
    }
    // styles
    let styles_str = print_styles(&page.styles, 1);
    if !styles_str.is_empty() {
        out.push_str(&format!("    styles: {{\n{}    }}\n", styles_str));
    }
    // state
    if !page.state.is_empty() {
        out.push_str(&print_state_block(&page.state, 1));
    }
    // children
    if !page.children.is_empty() {
        out.push_str("    children: [\n");
        for child in &page.children {
            out.push_str(&print_component_node(child, 2));
            out.push('\n');
        }
        out.push_str("    ]\n");
    }
    // inner functions
    let mut funcs: Vec<_> = page.functions.iter().collect();
    funcs.sort_by_key(|(k, _)| k.as_str());
    for (_, func) in &funcs {
        out.push_str(&print_function(func, 1));
        out.push('\n');
    }
    out.push('}');
    out
}

// ─── ComponentDef ─────────────────────────────────────────────────────────────

fn print_component_def(comp: &ComponentDef) -> String {
    let mut out = format!("component {}: {{\n", comp.name);

    if !comp.props.is_empty() {
        out.push_str("    props: {\n");
        let mut props: Vec<_> = comp.props.iter().collect();
        props.sort_by_key(|(k, _)| k.as_str());
        for (_, prop) in &props {
            let default_str = match &prop.default {
                Some(e) => format!(" = {}", print_expr(e)),
                None    => String::new(),
            };
            out.push_str(&format!("        {}: {}{}\n", prop.name, print_type(&prop.type_), default_str));
        }
        out.push_str("    }\n");
    }

    if !comp.state.is_empty() {
        out.push_str(&print_state_block(&comp.state, 1));
    }

    let styles_str = print_styles(&comp.styles, 1);
    if !styles_str.is_empty() {
        out.push_str(&format!("    styles: {{\n{}    }}\n", styles_str));
    }

    // events
    out.push_str(&print_event_map(&comp.events, 1));

    // animations
    for anim in &comp.animate {
        out.push_str(&print_animation(anim, 1));
    }

    if !comp.children.is_empty() {
        out.push_str("    children: [\n");
        for child in &comp.children {
            out.push_str(&print_component_node(child, 2));
            out.push('\n');
        }
        out.push_str("    ]\n");
    }

    // inner functions
    let mut funcs: Vec<_> = comp.functions.iter().collect();
    funcs.sort_by_key(|(k, _)| k.as_str());
    for (_, func) in &funcs {
        out.push_str(&print_function(func, 1));
        out.push('\n');
    }

    out.push('}');
    out
}

// ─── ComponentNode ────────────────────────────────────────────────────────────

fn print_component_node(node: &ComponentNode, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = format!("{}{}: {{\n", pad, node.kind);
    let inner_pad = "    ".repeat(indent + 1);

    // show_if
    if let Some(expr) = &node.show_if {
        out.push_str(&format!("{}show_if: {}\n", inner_pad, print_expr(expr)));
    }

    // data
    if let Some(expr) = &node.data {
        out.push_str(&format!("{}data: {}\n", inner_pad, print_expr(expr)));
    }

    // alignment (only print if non-default)
    if node.alignment != StackAlignment::TopLeft {
        out.push_str(&format!("{}alignment: {}\n", inner_pad, print_alignment(&node.alignment)));
    }

    // positioned
    if let Some(pos) = &node.positioned {
        out.push_str(&format!("{}positioned: {{\n", inner_pad));
        let deep_pad = "    ".repeat(indent + 2);
        if let Some(v) = &pos.top    { out.push_str(&format!("{}top: {}\n",    deep_pad, v)); }
        if let Some(v) = &pos.bottom { out.push_str(&format!("{}bottom: {}\n", deep_pad, v)); }
        if let Some(v) = &pos.left   { out.push_str(&format!("{}left: {}\n",   deep_pad, v)); }
        if let Some(v) = &pos.right  { out.push_str(&format!("{}right: {}\n",  deep_pad, v)); }
        if let Some(v) = &pos.width  { out.push_str(&format!("{}width: {}\n",  deep_pad, v)); }
        if let Some(v) = &pos.height { out.push_str(&format!("{}height: {}\n", deep_pad, v)); }
        out.push_str(&format!("{}}}\n", inner_pad));
    }

    // named props (content, src, value, title, icon, direction, current, validation, etc.)
    let prop_order = [
        // Original props
        "content", "src", "value", "title", "icon", "direction", "current", "validation",
        // New first-class props
        "message", "placeholder", "url", "lat", "lng", "min", "max",
        "lines", "length", "count", "selected", "checked", "refreshing",
        "duration", "label", "text",
        // New event props stored as props
        "on_scan", "on_complete", "on_refresh", "on_increment", "on_decrement",
        "on_long_press", "on_drag", "on_swipe",
    ];
    for key in &prop_order {
        if let Some(expr) = node.props.get(*key) {
            out.push_str(&format!("{}{}: {}\n", inner_pad, key, print_expr(expr)));
        }
    }

    // remaining props not in the named set
    let mut other_props: Vec<_> = node.props.iter()
        .filter(|(k, _)| !prop_order.contains(&k.as_str()))
        .collect();
    other_props.sort_by_key(|(k, _)| k.as_str());
    for (key, expr) in other_props {
        out.push_str(&format!("{}{}: {}\n", inner_pad, key, print_expr(expr)));
    }

    // styles
    let styles_str = print_styles(&node.styles, indent + 1);
    if !styles_str.is_empty() {
        out.push_str(&format!("{}styles: {{\n{}{}}}\n", inner_pad, styles_str, inner_pad));
    }

    // events
    out.push_str(&print_event_map(&node.events, indent + 1));

    // animations
    for anim in &node.animate {
        out.push_str(&print_animation(anim, indent + 1));
    }

    // build prop
    if let Some(build) = &node.build {
        if !build.params.is_empty() {
            let param_name = &build.params[0].0;
            out.push_str(&format!("{}build: ({}) => {{\n", inner_pad, param_name));
            for stmt in &build.body {
                out.push_str(&print_stmt(stmt, indent + 2));
            }
            out.push_str(&format!("{}}}\n", inner_pad));
        }
    }

    // children
    if !node.children.is_empty() {
        out.push_str(&format!("{}children: [\n", inner_pad));
        for child in &node.children {
            out.push_str(&print_component_node(child, indent + 2));
            out.push('\n');
        }
        out.push_str(&format!("{}]\n", inner_pad));
    }

    out.push_str(&format!("{}}}", pad));
    out
}

fn print_alignment(a: &StackAlignment) -> &'static str {
    match a {
        StackAlignment::TopLeft      => "top_left",
        StackAlignment::TopCenter    => "top_center",
        StackAlignment::TopRight     => "top_right",
        StackAlignment::CenterLeft   => "center_left",
        StackAlignment::Center       => "center",
        StackAlignment::CenterRight  => "center_right",
        StackAlignment::BottomLeft   => "bottom_left",
        StackAlignment::BottomCenter => "bottom_center",
        StackAlignment::BottomRight  => "bottom_right",
    }
}

// ─── Styles ───────────────────────────────────────────────────────────────────

/// Returns the inner lines of a styles block (without the outer braces).
/// Returns empty string if there is nothing to print.
fn print_styles(s: &Styles, indent: usize) -> String {
    let pad = "    ".repeat(indent + 1);
    let mut lines = String::new();

    macro_rules! opt_str {
        ($field:expr, $key:expr) => {
            if let Some(v) = &$field {
                lines.push_str(&format!("{}{}: {}\n", pad, $key, format_style_value(v)));
            }
        };
    }

    opt_str!(s.width,          "width");
    opt_str!(s.height,         "height");
    opt_str!(s.min_width,      "min_width");
    opt_str!(s.max_width,      "max_width");
    opt_str!(s.min_height,     "min_height");
    opt_str!(s.max_height,     "max_height");
    opt_str!(s.x,              "x");
    opt_str!(s.y,              "y");
    opt_str!(s.flex,           "flex");
    opt_str!(s.flex_wrap,      "flex_wrap");
    opt_str!(s.direction,      "direction");
    opt_str!(s.align,          "align");
    opt_str!(s.justify,        "justify");
    opt_str!(s.gap,            "gap");
    opt_str!(s.aspect_ratio,   "aspect_ratio");
    opt_str!(s.margin,         "margin");
    opt_str!(s.margin_top,     "margin_top");
    opt_str!(s.margin_bottom,  "margin_bottom");
    opt_str!(s.margin_left,    "margin_left");
    opt_str!(s.margin_right,   "margin_right");
    opt_str!(s.padding,        "padding");
    opt_str!(s.padding_top,    "padding_top");
    opt_str!(s.padding_bottom, "padding_bottom");
    opt_str!(s.padding_left,   "padding_left");
    opt_str!(s.padding_right,  "padding_right");
    opt_str!(s.background,     "background");
    opt_str!(s.color,          "color");
    opt_str!(s.font_size,      "font_size");
    opt_str!(s.font_weight,    "font_weight");
    opt_str!(s.font_family,    "font_family");
    opt_str!(s.border,         "border");
    opt_str!(s.border_radius,  "border_radius");
    opt_str!(s.opacity,        "opacity");

    if let Some(v) = s.visible {
        lines.push_str(&format!("{}visible: {}\n", pad, v));
    }
    if let Some(v) = s.safe_area {
        lines.push_str(&format!("{}safe_area: {}\n", pad, v));
    }

    // overflow — skip default (Visible)
    if s.overflow != OverflowValue::Visible {
        lines.push_str(&format!("{}overflow: {}\n", pad, print_overflow_value(&s.overflow)));
    }
    if let Some(v) = &s.overflow_x {
        lines.push_str(&format!("{}overflow_x: {}\n", pad, print_overflow_value(v)));
    }
    if let Some(v) = &s.overflow_y {
        lines.push_str(&format!("{}overflow_y: {}\n", pad, print_overflow_value(v)));
    }

    // clip_behavior — skip default (AntiAliased)
    if s.clip_behavior != ClipBehavior::AntiAliased {
        lines.push_str(&format!("{}clip_behavior: {}\n", pad, print_clip_behavior(&s.clip_behavior)));
    }

    // text_overflow — skip default (Clip)
    if s.text_overflow != TextOverflowValue::Clip {
        lines.push_str(&format!("{}text_overflow: {}\n", pad, print_text_overflow(&s.text_overflow)));
    }
    if let Some(v) = s.max_lines {
        lines.push_str(&format!("{}max_lines: {}\n", pad, v));
    }
    if let Some(v) = s.line_clamp {
        lines.push_str(&format!("{}line_clamp: {}\n", pad, v));
    }

    // image_fit — skip default (Contain)
    if s.image_fit != ImageFitValue::Contain {
        lines.push_str(&format!("{}fit: {}\n", pad, print_image_fit(&s.image_fit)));
    }

    if let Some(v) = s.scroll_indicator {
        lines.push_str(&format!("{}scroll_indicator: {}\n", pad, v));
    }

    // scroll_snap — skip default (None_)
    if s.scroll_snap != ScrollSnap::None_ {
        lines.push_str(&format!("{}scroll_snap: {}\n", pad, print_scroll_snap(&s.scroll_snap)));
    }

    opt_str!(s.scroll_enabled, "scroll_enabled");
    opt_str!(s.on_scroll,      "on_scroll");
    opt_str!(s.on_scroll_end,  "on_scroll_end");

    // Extra props
    let mut extra: Vec<_> = s.extra.iter().collect();
    extra.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in &extra {
        lines.push_str(&format!("{}{}: {}\n", pad, k, format_style_value(v)));
    }

    // Breakpoint overrides
    let mut bps: Vec<_> = s.breakpoint_overrides.iter().collect();
    bps.sort_by_key(|(k, _)| k.as_str());
    let bp_pad = "    ".repeat(indent + 1);
    for (bp_name, bp_styles) in &bps {
        let inner = print_styles(bp_styles, indent + 1);
        if !inner.is_empty() {
            lines.push_str(&format!("{}@{} {{\n", bp_pad, bp_name));
            // inner has indent+2 already, but for breakpoint_override in grammar
            // the inner props are generic_style_prop at one level deep
            lines.push_str(&inner);
            lines.push_str(&format!("{}}}\n", bp_pad));
        }
    }

    lines
}

fn print_overflow_value(v: &OverflowValue) -> &'static str {
    match v {
        OverflowValue::Visible  => "visible",
        OverflowValue::Hidden   => "hidden",
        OverflowValue::Scroll   => "scroll",
        OverflowValue::ScrollX  => "scroll_x",
        OverflowValue::ScrollY  => "scroll_y",
        OverflowValue::Auto     => "auto",
    }
}

fn print_text_overflow(v: &TextOverflowValue) -> &'static str {
    match v {
        TextOverflowValue::Clip     => "clip",
        TextOverflowValue::Ellipsis => "ellipsis",
        TextOverflowValue::Fade     => "fade",
    }
}

fn print_image_fit(v: &ImageFitValue) -> &'static str {
    match v {
        ImageFitValue::Cover     => "cover",
        ImageFitValue::Contain   => "contain",
        ImageFitValue::Fill      => "fill",
        ImageFitValue::None_     => "none",
        ImageFitValue::ScaleDown => "scale_down",
    }
}

fn print_clip_behavior(v: &ClipBehavior) -> &'static str {
    match v {
        ClipBehavior::None_      => "none",
        ClipBehavior::Hard       => "hard",
        ClipBehavior::AntiAliased => "anti_aliased",
    }
}

fn print_scroll_snap(v: &ScrollSnap) -> &'static str {
    match v {
        ScrollSnap::None_  => "none",
        ScrollSnap::Start  => "start",
        ScrollSnap::Center => "center",
        ScrollSnap::End    => "end",
    }
}

// ─── EventMap ─────────────────────────────────────────────────────────────────

fn print_event_map(events: &EventMap, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();
    macro_rules! emit_event {
        ($field:expr, $name:expr) => {
            if let Some(expr) = &$field {
                out.push_str(&format!("{}{}: {}\n", pad, $name, print_expr(expr)));
            }
        };
    }
    emit_event!(events.on_click,       "on_click");
    emit_event!(events.on_change,      "on_change");
    emit_event!(events.on_submit,      "on_submit");
    emit_event!(events.on_select,      "on_select");
    emit_event!(events.on_touch_start, "on_touch_start");
    emit_event!(events.on_touch_move,  "on_touch_move");
    emit_event!(events.on_touch_end,   "on_touch_end");
    emit_event!(events.on_mount,       "on_mount");
    emit_event!(events.on_update,      "on_update");
    emit_event!(events.watch,          "watch");
    emit_event!(events.on_unmount,     "on_unmount");
    out
}

// ─── Animation ────────────────────────────────────────────────────────────────

fn print_animation(anim: &Animation, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let inner = "    ".repeat(indent + 1);
    let mut out = format!("{}animate: {{\n", pad);
    out.push_str(&format!("{}property: \"{}\"\n", inner, anim.property));
    out.push_str(&format!("{}from: {}\n", inner, anim.from));
    out.push_str(&format!("{}to: {}\n", inner, anim.to));
    out.push_str(&format!("{}duration: {}ms\n", inner, anim.duration_ms));
    if anim.delay_ms > 0 {
        out.push_str(&format!("{}delay: {}ms\n", inner, anim.delay_ms));
    }
    out.push_str(&format!("{}easing: {}\n", inner, print_easing(&anim.easing)));
    if anim.repeat {
        out.push_str(&format!("{}repeat: true\n", inner));
    }
    if anim.auto_reverse {
        out.push_str(&format!("{}auto_reverse: true\n", inner));
    }
    out.push_str(&format!("{}}}\n", pad));
    out
}

fn print_easing(e: &EasingType) -> &'static str {
    match e {
        EasingType::Linear    => "linear",
        EasingType::EaseIn    => "ease_in",
        EasingType::EaseOut   => "ease_out",
        EasingType::EaseInOut => "ease_in_out",
        EasingType::Bounce    => "bounce",
        EasingType::Spring    => "spring",
    }
}

// ─── State block ──────────────────────────────────────────────────────────────

fn print_state_block(state: &std::collections::HashMap<String, StateField>, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let inner = "    ".repeat(indent + 1);
    let mut out = format!("{}state: {{\n", pad);
    let mut fields: Vec<_> = state.iter().collect();
    fields.sort_by_key(|(k, _)| k.as_str());
    for (_, field) in &fields {
        let default_str = match &field.default {
            Some(e) => format!(" = {}", print_expr(e)),
            None    => String::new(),
        };
        out.push_str(&format!("{}{}: {}{}\n", inner, field.name, print_type(&field.type_), default_str));
    }
    out.push_str(&format!("{}}}\n", pad));
    out
}

// ─── Expr ─────────────────────────────────────────────────────────────────────

pub fn print_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(v)              => print_value(v),
        Expr::Var(name)               => name.clone(),
        Expr::StateField(f)           => format!("state.{}", f),
        Expr::StoreField(store, field) => format!("{}.{}", store, field),
        Expr::BinOp(l, op, r)        => format!("{} {} {}", print_expr(l), print_op(op), print_expr(r)),
        Expr::Call(ce)                => print_call_expr(ce),
        Expr::NullCoalesce(l, r)      => format!("{} ?? {}", print_expr(l), print_expr(r)),
        Expr::SafeNav(parts)          => parts.join("?."),
        Expr::MethodCall(recv, method, args) => {
            let args_str = args.iter().map(print_expr).collect::<Vec<_>>().join(", ");
            format!("{}.{}({})", print_expr(recv), method, args_str)
        }
        Expr::Lambda(params, body) => {
            let params_str = params.join(", ");
            let mut body_str = String::new();
            for stmt in body {
                body_str.push_str(&format!(" {} ", print_stmt_inline(stmt)));
            }
            format!("({}) => {{{}}}", params_str, body_str)
        }
        Expr::Interpolated(segments) => {
            use crate::parser::ast::InterpolatedSegment;
            let inner: String = segments.iter().map(|seg| match seg {
                InterpolatedSegment::Literal(s) => s.clone(),
                InterpolatedSegment::Expr(e)    => format!("\\({})", print_expr(e)),
            }).collect();
            format!("\"{}\"", inner)
        }
        // ── Navigation expressions ─────────────────────────────────────────
        Expr::Navigate(route, opts) => {
            let r = print_expr(route);
            let mut parts: Vec<String> = Vec::new();
            if opts.replace     { parts.push("replace: true".to_string()); }
            if opts.clear_stack { parts.push("clear_stack: true".to_string()); }
            if opts.single_top  { parts.push("single_top: true".to_string()); }
            if let Some(t) = &opts.transition { parts.push(format!("transition: \"{}\"", t)); }
            if parts.is_empty() {
                format!("navigate({})", r)
            } else {
                format!("navigate({}, {})", r, parts.join(", "))
            }
        }
        Expr::NavigateReplace(route)  => format!("navigate_replace({})", print_expr(route)),
        Expr::NavigateBack            => "navigate_back()".to_string(),
        Expr::NavigateBackTo(route)   => format!("navigate_back_to({})", print_expr(route)),
        Expr::NavigateModal(route)    => format!("navigate_modal({})", print_expr(route)),
        Expr::NavigateDismiss         => "navigate_dismiss()".to_string(),
    }
}

fn print_value(v: &Value) -> String {
    match v {
        Value::Str(s)    => format!("\"{}\"", s),
        Value::Int(n)    => n.to_string(),
        Value::Float(f)  => format_float(*f),
        Value::Bool(b)   => b.to_string(),
        Value::Null      => "null".to_string(),
        Value::List(items) => {
            let inner = items.iter().map(print_value).collect::<Vec<_>>().join(", ");
            format!("[{}]", inner)
        }
        Value::Object(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| k.as_str());
            let inner = entries.iter()
                .map(|(k, v)| format!("{}: {}", k, print_value(v)))
                .collect::<Vec<_>>().join(", ");
            format!("{{ {} }}", inner)
        }
    }
}

fn print_call_expr(ce: &CallExpr) -> String {
    let mut parts: Vec<String> = ce.args.iter().map(print_expr).collect();
    for (k, v) in &ce.named_args {
        parts.push(format!("{}: {}", k, print_expr(v)));
    }
    let args_str = parts.join(", ");
    if ce.func == "navigate" || ce.func == "navigate_back" {
        format!("{}({})", ce.func, args_str)
    } else if ce.func.starts_with("wait:") {
        format!("{}({})", ce.func, args_str)
    } else {
        format!("{}:({})", ce.func, args_str)
    }
}

fn print_op(op: &Op) -> &'static str {
    match op {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Mul => "*",
        Op::Div => "/",
        Op::Mod => "%",
        Op::Eq  => "==",
        Op::Ne  => "!=",
        Op::Lt  => "<",
        Op::Le  => "<=",
        Op::Gt  => ">",
        Op::Ge  => ">=",
        Op::And => "&&",
        Op::Or  => "||",
        Op::Not => "!",
    }
}

// ─── Stmt ─────────────────────────────────────────────────────────────────────

fn print_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::VarDecl(vd) => {
            let mut_str = if vd.mutable { "mut " } else { "" };
            let type_part = vd.type_.as_ref().map_or_else(String::new, |t| format!(": {}", print_type(t)));
            match &vd.initializer {
                Some(init) => format!("{}:var {}{} = {}\n", pad, mut_str, vd.name, print_expr(init)),
                None => format!("{}:var {}{}{}\n", pad, mut_str, vd.name, type_part),
            }
        }
        Stmt::Assign(name, expr) => {
            format!("{}{} = {}\n", pad, name, print_expr(expr))
        }
        Stmt::Return(expr) => {
            format!("{}return {}\n", pad, print_expr(expr))
        }
        Stmt::Call(ce) => {
            let args_str = ce.args.iter().map(print_expr).collect::<Vec<_>>().join(", ");
            format!("{}{}:({})\n", pad, ce.func, args_str)
        }
        Stmt::Wait(ce) => {
            let args_str = ce.args.iter().map(print_expr).collect::<Vec<_>>().join(", ");
            format!("{}wait:{}({})\n", pad, ce.func, args_str)
        }
        Stmt::WaitFetch(fe) => {
            let url_str = print_expr(&fe.url);
            format!("{}result = wait:fetch({}, {{ method: \"{}\" }})\n", pad, url_str, fe.method)
        }
        Stmt::If(cond, then_body, else_body) => {
            let mut out = format!("{}if {} {{\n", pad, print_expr(cond));
            for s in then_body {
                out.push_str(&print_stmt(s, indent + 1));
            }
            out.push_str(&format!("{}}}", pad));
            if let Some(else_stmts) = else_body {
                out.push_str(" else {\n");
                for s in else_stmts {
                    out.push_str(&print_stmt(s, indent + 1));
                }
                out.push_str(&format!("{}}}", pad));
            }
            out.push('\n');
            out
        }
        Stmt::For(var, iter, body) => {
            let mut out = format!("{}for {} in {} {{\n", pad, var, print_expr(iter));
            for s in body {
                out.push_str(&print_stmt(s, indent + 1));
            }
            out.push_str(&format!("{}}}\n", pad));
            out
        }
        Stmt::Switch(disc, cases) => {
            let mut out = format!("{}switch {} {{\n", pad, print_expr(disc));
            for (val, body) in cases {
                out.push_str(&format!("{}    case {} => {{\n", pad, print_expr(val)));
                for s in body {
                    out.push_str(&print_stmt(s, indent + 2));
                }
                out.push_str(&format!("{}    }}\n", pad));
            }
            out.push_str(&format!("{}}}\n", pad));
            out
        }
        Stmt::TryCatch { body, catch_param, catch_body, finally_body } => {
            let mut out = format!("{}try {{\n", pad);
            for s in body {
                out.push_str(&print_stmt(s, indent + 1));
            }
            out.push_str(&format!("{}}} catch ({}) {{\n", pad, catch_param));
            for s in catch_body {
                out.push_str(&print_stmt(s, indent + 1));
            }
            out.push_str(&format!("{}}}", pad));
            if let Some(finally_stmts) = finally_body {
                out.push_str(" finally {\n");
                for s in finally_stmts {
                    out.push_str(&print_stmt(s, indent + 1));
                }
                out.push_str(&format!("{}}}", pad));
            }
            out.push('\n');
            out
        }
        Stmt::PluginCall(pc) => {
            let params_str: String = pc.params.iter()
                .map(|(k, v)| format!("{}: {}", k, print_expr(v)))
                .collect::<Vec<_>>()
                .join("  ");
            format!(
                "{}plugin: {{ name: \"{}\"  method: {}  params: {{ {} }} }}\n",
                pad, pc.plugin_name, pc.method, params_str
            )
        }
    }
}

/// Inline stmt for lambdas (no trailing newline)
fn print_stmt_inline(stmt: &Stmt) -> String {
    let s = print_stmt(stmt, 0);
    s.trim_end_matches('\n').to_string()
}

// ─── Test suite ───────────────────────────────────────────────────────────────

fn print_test_suite(suite: &TestSuite) -> String {
    let mut out = format!("describe: \"{}\" => {{\n", suite.name);
    for case in &suite.cases {
        out.push_str(&format!("    it: \"{}\" => {{\n", case.name));
        // mocks
        for mock in &case.mocks {
            out.push_str("        mock: {\n");
            out.push_str(&format!("            url: \"{}\"\n", mock.url_pattern));
            out.push_str(&format!("            status: {}\n", mock.status_code));
            out.push_str("        }\n");
        }
        // body stmts
        for stmt in &case.body {
            out.push_str(&print_stmt(stmt, 2));
        }
        // assertions
        for assertion in &case.assertions {
            out.push_str(&format!("        expect: {} {}\n",
                print_expr(&assertion.expr),
                print_matcher(&assertion.matcher, &assertion.expected)));
        }
        out.push_str("    }\n");
    }
    out.push('}');
    out
}

fn print_matcher(matcher: &Matcher, expected: &Option<Expr>) -> String {
    match matcher {
        Matcher::ToBe      => format!(".toBe:({})", expected.as_ref().map(print_expr).unwrap_or_default()),
        Matcher::ToEqual   => format!(".toEqual:({})", expected.as_ref().map(print_expr).unwrap_or_default()),
        Matcher::ToContain => format!(".toContain:({})", expected.as_ref().map(print_expr).unwrap_or_default()),
        Matcher::ToBeNull  => ".toBeNull:()".to_string(),
        Matcher::ToBeTrue  => ".toBeTrue:()".to_string(),
        Matcher::ToBeFalse => ".toBeFalse:()".to_string(),
        Matcher::ToThrow   => ".toThrow:()".to_string(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_source_str;

    // ── Vars round-trip ───────────────────────────────────────────────────────

    #[test]
    fn round_trip_vars() {
        let mut ast = AST::default();
        ast.vars.insert("primary".to_string(), "#007BFF".to_string());
        ast.vars.insert("spacing".to_string(), "10dp".to_string());
        let printed = print(&ast);
        eprintln!("vars printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert!(reparsed.vars.contains_key("primary"), "missing primary in reparsed vars");
        assert!(reparsed.vars.contains_key("spacing"), "missing spacing in reparsed vars");
    }

    // ── I18n round-trip ───────────────────────────────────────────────────────

    #[test]
    fn round_trip_i18n() {
        let mut ast = AST::default();
        ast.i18n.insert("welcome".to_string(), "Welcome".to_string());
        ast.i18n.insert("logout".to_string(), "Log out".to_string());
        let printed = print(&ast);
        eprintln!("i18n printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.i18n.get("welcome"), Some(&"Welcome".to_string()));
        assert_eq!(reparsed.i18n.get("logout"), Some(&"Log out".to_string()));
    }

    // ── Imports round-trip ────────────────────────────────────────────────────

    #[test]
    fn round_trip_imports() {
        let mut ast = AST::default();
        ast.imports.push(Import {
            names: vec![("AppBar".to_string(), None)],
            path: "frame-core".to_string(),
        });
        ast.imports.push(Import {
            names: vec![("Card".to_string(), Some("MyCard".to_string()))],
            path: "./component/Card.fr".to_string(),
        });
        let printed = print(&ast);
        eprintln!("imports printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.imports.len(), 2);
        let appbar_import = reparsed.imports.iter().find(|i| i.path == "frame-core").unwrap();
        assert_eq!(appbar_import.names[0].0, "AppBar");
        assert!(appbar_import.names[0].1.is_none());
    }

    // ── Consts round-trip ─────────────────────────────────────────────────────

    #[test]
    fn round_trip_consts() {
        let mut ast = AST::default();
        ast.consts.insert("author".to_string(), ConstValue::Str("john doe".to_string()));
        ast.consts.insert("max_items".to_string(), ConstValue::Int(10));
        ast.consts.insert("debug".to_string(), ConstValue::Bool(true));
        let printed = print(&ast);
        eprintln!("consts printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert!(reparsed.consts.contains_key("author"));
        assert!(reparsed.consts.contains_key("max_items"));
        assert!(reparsed.consts.contains_key("debug"));
        match reparsed.consts.get("author") {
            Some(ConstValue::Str(s)) => assert_eq!(s, "john doe"),
            _ => panic!("expected string const"),
        }
        match reparsed.consts.get("max_items") {
            Some(ConstValue::Int(n)) => assert_eq!(*n, 10),
            _ => panic!("expected int const"),
        }
    }

    // ── Breakpoints round-trip ────────────────────────────────────────────────

    #[test]
    fn round_trip_breakpoints() {
        let mut ast = AST::default();
        ast.breakpoints.push(Breakpoint { name: "sm".to_string(), min_width_dp: 360.0 });
        ast.breakpoints.push(Breakpoint { name: "md".to_string(), min_width_dp: 600.0 });
        let printed = print(&ast);
        eprintln!("breakpoints printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.breakpoints.len(), 2);
        assert!(reparsed.breakpoints.iter().any(|b| b.name == "sm"));
        assert!(reparsed.breakpoints.iter().any(|b| b.name == "md"));
    }

    // ── Page round-trip ───────────────────────────────────────────────────────

    #[test]
    fn round_trip_page() {
        let mut ast = AST::default();
        ast.pages.push(Page {
            name: "Home".to_string(),
            route: "/".to_string(),
            ..Default::default()
        });
        let printed = print(&ast);
        eprintln!("page printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.pages.len(), 1);
        assert_eq!(reparsed.pages[0].name, "Home");
        assert_eq!(reparsed.pages[0].route, "/");
    }

    // ── Page with styles round-trip ───────────────────────────────────────────

    #[test]
    fn round_trip_page_with_styles() {
        let mut ast = AST::default();
        let mut styles = Styles::default();
        styles.width = Some("100%".to_string());
        styles.overflow = OverflowValue::Scroll;
        styles.background = Some("#FFFFFF".to_string());
        ast.pages.push(Page {
            name: "Styled".to_string(),
            route: "/styled".to_string(),
            styles,
            ..Default::default()
        });
        let printed = print(&ast);
        eprintln!("page_styles printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.pages.len(), 1);
        let page = &reparsed.pages[0];
        assert_eq!(page.styles.width.as_deref(), Some("100%"));
        assert_eq!(page.styles.overflow, OverflowValue::Scroll);
    }

    // ── Page with children round-trip ─────────────────────────────────────────

    #[test]
    fn round_trip_page_with_children() {
        let mut ast = AST::default();
        let mut child = ComponentNode::default();
        child.kind = "text".to_string();
        child.props.insert("content".to_string(), Expr::Literal(Value::Str("Hello".to_string())));
        ast.pages.push(Page {
            name: "WithChildren".to_string(),
            route: "/children".to_string(),
            children: vec![child],
            ..Default::default()
        });
        let printed = print(&ast);
        eprintln!("page_children printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.pages[0].children.len(), 1);
        assert_eq!(reparsed.pages[0].children[0].kind, "text");
    }

    // ── Store round-trip ──────────────────────────────────────────────────────

    #[test]
    fn round_trip_store() {
        let mut ast = AST::default();
        let mut store = StoreSlice {
            name: "AuthStore".to_string(),
            ..Default::default()
        };
        store.fields.insert("token".to_string(), StoreField {
            name: "token".to_string(),
            type_: FRType::String_,
            default: Some(Expr::Literal(Value::Str("".to_string()))),
        });
        store.fields.insert("is_loading".to_string(), StoreField {
            name: "is_loading".to_string(),
            type_: FRType::Bool,
            default: Some(Expr::Literal(Value::Bool(false))),
        });
        store.persist.insert("token".to_string(), PersistStrategy::Secure);
        ast.stores.insert("AuthStore".to_string(), store);
        let printed = print(&ast);
        eprintln!("store printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert!(reparsed.stores.contains_key("AuthStore"));
        let s = &reparsed.stores["AuthStore"];
        assert!(s.fields.contains_key("token"));
        assert!(s.fields.contains_key("is_loading"));
        assert_eq!(s.persist.get("token"), Some(&PersistStrategy::Secure));
    }

    // ── Component def round-trip ──────────────────────────────────────────────

    #[test]
    fn round_trip_component_def() {
        let mut ast = AST::default();
        let mut comp = ComponentDef {
            name: "Card".to_string(),
            ..Default::default()
        };
        comp.props.insert("title".to_string(), PropDef {
            name: "title".to_string(),
            type_: FRType::String_,
            required: true,
            default: None,
        });
        comp.props.insert("count".to_string(), PropDef {
            name: "count".to_string(),
            type_: FRType::Int,
            required: false,
            default: Some(Expr::Literal(Value::Int(0))),
        });
        let mut child = ComponentNode::default();
        child.kind = "text".to_string();
        child.props.insert("content".to_string(), Expr::Var("$title".to_string()));
        comp.children.push(child);
        ast.components.insert("Card".to_string(), comp);
        let printed = print(&ast);
        eprintln!("component printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert!(reparsed.components.contains_key("Card"));
        let card = &reparsed.components["Card"];
        assert!(card.props.contains_key("title"));
        assert!(card.props.contains_key("count"));
        assert_eq!(card.children.len(), 1);
    }

    // ── Function round-trip ───────────────────────────────────────────────────

    #[test]
    fn round_trip_function() {
        let mut ast = AST::default();
        ast.functions.insert("greet".to_string(), Function {
            name: "greet".to_string(),
            is_async: false,
            params: vec![("name".to_string(), FRType::String_, None)],
            return_type: None,
            body: vec![
                Stmt::Assign("x".to_string(), Expr::Var("name".to_string())),
            ],
        });
        let printed = print(&ast);
        eprintln!("function printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert!(reparsed.functions.contains_key("greet"));
        let f = &reparsed.functions["greet"];
        assert!(!f.is_async);
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].0, "name");
    }

    // ── Test suite round-trip ─────────────────────────────────────────────────

    #[test]
    fn round_trip_test_suite() {
        let mut ast = AST::default();
        ast.tests.push(TestSuite {
            name: "Suite".to_string(),
            cases: vec![TestCase {
                name: "test case".to_string(),
                mocks: vec![],
                body: vec![],
                assertions: vec![Assertion {
                    expr: Expr::Literal(Value::Bool(true)),
                    matcher: Matcher::ToBeTrue,
                    expected: None,
                }],
            }],
        });
        let printed = print(&ast);
        eprintln!("test_suite printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert_eq!(reparsed.tests.len(), 1);
        assert_eq!(reparsed.tests[0].name, "Suite");
        assert_eq!(reparsed.tests[0].cases.len(), 1);
        assert_eq!(reparsed.tests[0].cases[0].name, "test case");
    }

    // ── Typography round-trip ─────────────────────────────────────────────────

    #[test]
    fn round_trip_typography() {
        let mut ast = AST::default();
        ast.typography.insert("headline".to_string(), TypographyStyle {
            name: "headline".to_string(),
            font_size: "24sp".to_string(),
            font_weight: Some("bold".to_string()),
            font_family: None,
            line_height: None,
            letter_spacing: None,
            color: None,
            breakpoint_overrides: std::collections::HashMap::new(),
        });
        let printed = print(&ast);
        eprintln!("typography printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        assert!(reparsed.typography.contains_key("headline"));
        let ts = &reparsed.typography["headline"];
        assert_eq!(ts.font_size, "24sp");
        assert_eq!(ts.font_weight.as_deref(), Some("bold"));
    }

    // ── Styles with breakpoint overrides round-trip ───────────────────────────

    #[test]
    fn round_trip_styles_breakpoint_override() {
        let mut ast = AST::default();
        let mut styles = Styles::default();
        styles.width = Some("100%".to_string());
        let mut bp_styles = Styles::default();
        bp_styles.width = Some("75%".to_string());
        styles.breakpoint_overrides.insert("md".to_string(), Box::new(bp_styles));
        ast.pages.push(Page {
            name: "BpPage".to_string(),
            route: "/bp".to_string(),
            styles,
            ..Default::default()
        });
        let printed = print(&ast);
        eprintln!("styles_bp printed:\n{}", printed);
        let reparsed = parse_source_str(&printed).expect("round-trip parse failed");
        let page = &reparsed.pages[0];
        assert_eq!(page.styles.width.as_deref(), Some("100%"));
        assert!(page.styles.breakpoint_overrides.contains_key("md"));
    }
}
