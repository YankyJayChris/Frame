//! Standard library injector for the Frame framework.
//!
//! Maps Frame built-in function calls to platform-native Kotlin/Swift code.

use crate::parser::ast::*;
use std::collections::HashMap;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Inject stdlib bindings into an AST.
/// Performs string interpolation resolution on string literals in the AST.
/// Code generators use emit_stdlib_call() directly for method translation.
pub fn inject(ast: AST) -> AST {
    let mut ast = ast;
    let vars = ast.vars.clone();
    for page in &mut ast.pages {
        resolve_vars_in_nodes(&mut page.children, &vars);
    }
    for comp in ast.components.values_mut() {
        resolve_vars_in_nodes(&mut comp.children, &vars);
    }
    ast
}

/// Emit a platform-specific code fragment for a stdlib call.
///
/// # Arguments
/// * `call` — dot-namespaced call: `"string.upper"`, `"list.filter"`, `"math.sqrt"`, etc.
/// * `platform` — `"android"` or `"ios"`
///
/// Returns the native code fragment to emit. Returns empty string for unknown calls.
pub fn emit_stdlib_call(call: &str, platform: &str) -> String {
    let is_android = platform == "android";
    // Strip args for base matching: "string.contains(x)" -> "string.contains"
    let base_call = if let Some(idx) = call.find('(') { &call[..idx] } else { call };

    match base_call {
        // ── String methods ────────────────────────────────────────────────
        "string.upper" => if is_android { ".uppercase()".into() } else { ".uppercased()".into() },
        "string.lower" => if is_android { ".lowercase()".into() } else { ".lowercased()".into() },
        "string.trim"  => if is_android { ".trim()".into() } else { ".trimmingCharacters(in: .whitespaces)".into() },
        "string.contains"   => if is_android { ".contains(x)".into() } else { ".contains(x)".into() },
        "string.starts_with" => if is_android { ".startsWith(x)".into() } else { ".hasPrefix(x)".into() },
        "string.ends_with"   => if is_android { ".endsWith(x)".into() } else { ".hasSuffix(x)".into() },
        "string.replace"     => if is_android { ".replaceFirst(a,b)".into() } else { ".replacingOccurrences(of:a,with:b)".into() },
        "string.replace_all" => if is_android { ".replace(a,b)".into() } else { ".replacingOccurrences(of:a,with:b)".into() },
        "string.split"  => if is_android { ".split(sep)".into() } else { ".components(separatedBy:sep)".into() },
        "string.join"   => if is_android { ".joinToString(sep)".into() } else { ".joined(separator:sep)".into() },
        "string.slice"  => if is_android { ".substring(s,e)".into() } else { "String(prefix(e)).dropFirst(s)".into() },
        "string.index_of" => if is_android { ".indexOf(x)".into() } else { ".range(of:x)".into() },
        "string.is_empty"  => if is_android { ".isEmpty()".into() } else { ".isEmpty".into() },
        "string.repeat"    => if is_android { ".repeat(n)".into() } else { "String(repeating:self,count:n)".into() },
        "string.pad_left"  => if is_android { ".padStart(n,c)".into() } else { "padLeft(n,c)".into() },
        "string.pad_right" => if is_android { ".padEnd(n,c)".into() } else { "padRight(n,c)".into() },
        "string.to_int"    => if is_android { ".toInt()!!".into() } else { "Int(self)!".into() },
        "string.to_float"  => if is_android { ".toDouble()!!".into() } else { "Double(self)!".into() },
        "string.to_bool"   => if is_android { ".toBooleanStrict()".into() } else { "Bool(self)!".into() },
        "string.length"    => if is_android { ".length".into() } else { ".count".into() },

        // ── Number functions ──────────────────────────────────────────────
        "number.abs"   => if is_android { "Math.abs(x)".into() } else { "abs(x)".into() },
        "number.ceil"  => if is_android { "Math.ceil(x.toDouble()).toInt()".into() } else { "Int(ceil(x))".into() },
        "number.floor" => if is_android { "Math.floor(x.toDouble()).toInt()".into() } else { "Int(floor(x))".into() },
        "number.round" => if is_android { "Math.round(x.toDouble()).toInt()".into() } else { "Int(round(x))".into() },
        "number.clamp" => if is_android { "x.coerceIn(lo,hi)".into() } else { "min(max(x,lo),hi)".into() },
        "number.max"   => if is_android { "maxOf(a,b)".into() } else { "max(a,b)".into() },
        "number.min"   => if is_android { "minOf(a,b)".into() } else { "min(a,b)".into() },
        "number.pow"   => if is_android { "x.toDouble().pow(exp)".into() } else { "pow(x,exp)".into() },
        "number.sqrt"  => if is_android { "Math.sqrt(x.toDouble())".into() } else { "sqrt(x)".into() },
        "number.is_nan" => if is_android { "x.isNaN()".into() } else { "x.isNaN".into() },
        "number.to_string" => if is_android { ".toString()".into() } else { "String(x)".into() },
        "number.random"    => if is_android { "Math.random()".into() } else { "Double.random(in:0..<1)".into() },
        "number.random_int" => if is_android { "(lo..hi).random()".into() } else { "Int.random(in:lo...hi)".into() },

        // ── List methods ──────────────────────────────────────────────────
        "list.length"   => if is_android { ".size".into() } else { ".count".into() },
        "list.push"     => if is_android { ".add(x)".into() } else { ".append(x)".into() },
        "list.pop"      => if is_android { ".removeLast()".into() } else { ".removeLast()".into() },
        "list.first"    => if is_android { ".first()".into() } else { ".first!".into() },
        "list.last"     => if is_android { ".last()".into() } else { ".last!".into() },
        "list.get"      => if is_android { ".get(i)".into() } else { "[i]".into() },
        "list.set"      => if is_android { ".set(i,v)".into() } else { "[i] = v".into() },
        "list.insert"   => if is_android { ".add(i,v)".into() } else { ".insert(v,at:i)".into() },
        "list.remove"   => if is_android { ".removeAt(i)".into() } else { ".remove(at:i)".into() },
        "list.contains" => if is_android { ".contains(x)".into() } else { ".contains(x)".into() },
        "list.index_of" => if is_android { ".indexOf(x)".into() } else { ".firstIndex(of:x)!".into() },
        "list.reverse"  => if is_android { ".reversed()".into() } else { ".reversed()".into() },
        "list.sort"     => if is_android { ".sorted()".into() } else { ".sorted()".into() },
        "list.sort_by"  => if is_android { ".sortedWith(compareBy{f(it)})".into() } else { ".sorted{f($0,$1)}".into() },
        "list.filter"   => if is_android { ".filter{p(it)}".into() } else { ".filter{p($0)}".into() },
        "list.map"      => if is_android { ".map{f(it)}".into() } else { ".map{f($0)}".into() },
        "list.reduce"   => if is_android { ".fold(i){acc,it->f(acc,it)}".into() } else { ".reduce(i){f($0,$1)}".into() },
        "list.find"     => if is_android { ".firstOrNull{p(it)}".into() } else { ".first{p($0)}".into() },
        "list.every"    => if is_android { ".all{p(it)}".into() } else { ".allSatisfy{p($0)}".into() },
        "list.any"      => if is_android { ".any{p(it)}".into() } else { ".contains{p($0)}".into() },
        "list.flat"     => if is_android { ".flatten()".into() } else { ".flatMap{$0}".into() },
        "list.flat_deep" => if is_android { ".flatMap{it.flatten()}".into() } else { ".flatMap{$0.flatMap{$0}}".into() },
        "list.slice"    => if is_android { ".subList(s,e)".into() } else { "Array(dropFirst(s).prefix(e-s))".into() },
        "list.distinct" => if is_android { ".distinct()".into() } else { "Array(Set(self))".into() },
        "list.count"    => if is_android { ".size".into() } else { ".count".into() },
        "list.sum"      => if is_android { ".sum()".into() } else { ".reduce(0,+)".into() },
        "list.average"  => if is_android { ".average()".into() } else { "reduce(0,+)/Double(count)".into() },
        "list.zip"      => if is_android { ".zip(other)".into() } else { "zip(self,other).map{[$0,$1]}".into() },
        "list.join"     => if is_android { ".joinToString(sep)".into() } else { ".joined(separator:sep)".into() },

        // ── Object methods ────────────────────────────────────────────────
        "object.keys"   => if is_android { ".keys.toList()".into() } else { "Array(dict.keys)".into() },
        "object.values" => if is_android { ".values.toList()".into() } else { "Array(dict.values)".into() },
        "object.has"    => if is_android { ".containsKey(k)".into() } else { "dict[k] != nil".into() },
        "object.get"    => if is_android { ".get(k)".into() } else { "dict[k]".into() },
        "object.set"    => if is_android { "[k] = v".into() } else { "dict[k] = v".into() },
        "object.remove" => if is_android { ".remove(k)".into() } else { "dict.removeValue(forKey:k)".into() },
        "object.merge"  => if is_android { ".putAll(other)".into() } else { "dict.merge(other){_,new in new}".into() },
        "object.to_json" => if is_android {
            "Gson().toJson(this)".into()
        } else {
            "String(data:JSONSerialization.data(withJSONObject:self)!,encoding:.utf8)!".into()
        },
        "from_json" => if is_android {
            "Gson().fromJson(s, Map::class.java)".into()
        } else {
            "JSONSerialization.jsonObject(with:s.data(using:.utf8)!)".into()
        },

        // ── Math namespace ────────────────────────────────────────────────
        "math.pi"    => if is_android { "Math.PI".into() } else { "Double.pi".into() },
        "math.e"     => if is_android { "Math.E".into() } else { "M_E".into() },
        "math.sqrt"  => if is_android { "Math.sqrt(x)".into() } else { "sqrt(x)".into() },
        "math.log"   => if is_android { "Math.log(x)".into() } else { "log(x)".into() },
        "math.log2"  => if is_android { "Math.log2(x)".into() } else { "log2(x)".into() },
        "math.log10" => if is_android { "Math.log10(x)".into() } else { "log10(x)".into() },
        "math.sin"   => if is_android { "Math.sin(x)".into() } else { "sin(x)".into() },
        "math.cos"   => if is_android { "Math.cos(x)".into() } else { "cos(x)".into() },
        "math.tan"   => if is_android { "Math.tan(x)".into() } else { "tan(x)".into() },
        "math.floor" => if is_android { "Math.floor(x)".into() } else { "floor(x)".into() },
        "math.ceil"  => if is_android { "Math.ceil(x)".into() } else { "ceil(x)".into() },
        "math.round" => if is_android { "Math.round(x)".into() } else { "round(x)".into() },
        "math.abs"   => if is_android { "Math.abs(x)".into() } else { "abs(x)".into() },
        "math.max"   => if is_android { "Math.max(a,b)".into() } else { "max(a,b)".into() },
        "math.min"   => if is_android { "Math.min(a,b)".into() } else { "min(a,b)".into() },
        "math.pow"   => if is_android { "Math.pow(x,n)".into() } else { "pow(x,n)".into() },

        // ── Date namespace ────────────────────────────────────────────────
        "date.now"            => if is_android { "System.currentTimeMillis()".into() } else { "Date().timeIntervalSince1970".into() },
        "date.from_timestamp" => if is_android { "Date(t)".into() } else { "Date(timeIntervalSince1970:t)".into() },
        "date.format"         => if is_android { "SimpleDateFormat(fmt).format(this)".into() } else { "DateFormatter().string(from:self)".into() },
        "date.diff"           => if is_android {
            "ChronoUnit.DAYS.between(this,other)".into()
        } else {
            "Calendar.current.dateComponents([.day],from:self,to:other).day!".into()
        },

        // ── Utility functions ──────────────────────────────────────────────
        "util.type_of"      => if is_android { "x::class.simpleName".into() } else { "type(of:x)".into() },
        "util.is_null"      => if is_android { "x == null".into() } else { "x == nil".into() },
        "util.is_not_null"  => if is_android { "x != null".into() } else { "x != nil".into() },
        "util.print"        => if is_android { "println(x)".into() } else { "print(x)".into() },
        "util.assert"       => if is_android { "assert(x)".into() } else { "assert(x)".into() },
        "util.uuid"         => if is_android { "UUID.randomUUID().toString()".into() } else { "UUID().uuidString".into() },
        "util.hash"         => if is_android { "x.hashCode().toString()".into() } else { "String(x.hashValue)".into() },
        "util.encode_base64" => if is_android {
            "Base64.getEncoder().encodeToString(x.toByteArray())".into()
        } else {
            "Data(x.utf8).base64EncodedString()".into()
        },
        "util.decode_base64" => if is_android {
            "String(Base64.getDecoder().decode(x))".into()
        } else {
            "String(data:Data(base64Encoded:x)!,encoding:.utf8)!".into()
        },
        "util.encode_url" => if is_android {
            "URLEncoder.encode(x, \"UTF-8\")".into()
        } else {
            "x.addingPercentEncoding(withAllowedCharacters:.urlQueryAllowed)!".into()
        },
        "util.decode_url" => if is_android {
            "URLDecoder.decode(x, \"UTF-8\")".into()
        } else {
            "x.removingPercentEncoding!".into()
        },

        // ── Null safety operators ─────────────────────────────────────────
        "null.coalesce" => if is_android { "?:".into() } else { "??".into() },
        "null.safe_nav" => "?.".into(),

        // Unknown call
        _ => String::new(),
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Resolve `$var` interpolation in a string using the given variable map.
/// Variables matching keys in `vars` are substituted; unknown refs are left as-is.
pub fn resolve_string_interpolation(s: &str, vars: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (k, v) in vars {
        result = result.replace(k.as_str(), v.as_str());
    }
    result
}

/// Walk a slice of component nodes, resolving string interpolation in their props.
fn resolve_vars_in_nodes(nodes: &mut Vec<ComponentNode>, vars: &HashMap<String, String>) {
    for node in nodes.iter_mut() {
        resolve_vars_in_exprs_map(&mut node.props, vars);
        resolve_vars_in_nodes(&mut node.children, vars);
    }
}

/// Apply string interpolation to all string-literal Expr values in a props map.
fn resolve_vars_in_exprs_map(props: &mut std::collections::HashMap<String, Expr>, vars: &HashMap<String, String>) {
    for expr in props.values_mut() {
        resolve_vars_in_expr(expr, vars);
    }
}

/// Recursively resolve string interpolation in an expression.
fn resolve_vars_in_expr(expr: &mut Expr, vars: &HashMap<String, String>) {
    match expr {
        Expr::Literal(Value::Str(s)) => {
            *s = resolve_string_interpolation(s, vars);
        }
        Expr::BinOp(left, _, right) => {
            resolve_vars_in_expr(left, vars);
            resolve_vars_in_expr(right, vars);
        }
        Expr::Call(call) => {
            for arg in &mut call.args {
                resolve_vars_in_expr(arg, vars);
            }
        }
        Expr::NullCoalesce(a, b) => {
            resolve_vars_in_expr(a, vars);
            resolve_vars_in_expr(b, vars);
        }
        Expr::MethodCall(recv, _, args) => {
            resolve_vars_in_expr(recv, vars);
            for arg in args.iter_mut() {
                resolve_vars_in_expr(arg, vars);
            }
        }
        _ => {}
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── String method mappings ────────────────────────────────────────────
    #[test]
    fn test_emit_string_upper_android() {
        assert_eq!(emit_stdlib_call("string.upper", "android"), ".uppercase()");
    }
    #[test]
    fn test_emit_string_upper_ios() {
        assert_eq!(emit_stdlib_call("string.upper", "ios"), ".uppercased()");
    }
    #[test]
    fn test_emit_string_lower_android() {
        assert_eq!(emit_stdlib_call("string.lower", "android"), ".lowercase()");
    }
    #[test]
    fn test_emit_string_lower_ios() {
        assert_eq!(emit_stdlib_call("string.lower", "ios"), ".lowercased()");
    }
    #[test]
    fn test_emit_string_trim_android() {
        assert_eq!(emit_stdlib_call("string.trim", "android"), ".trim()");
    }
    #[test]
    fn test_emit_string_trim_ios() {
        assert_eq!(emit_stdlib_call("string.trim", "ios"), ".trimmingCharacters(in: .whitespaces)");
    }
    #[test]
    fn test_emit_string_is_empty_android() {
        assert_eq!(emit_stdlib_call("string.is_empty", "android"), ".isEmpty()");
    }
    #[test]
    fn test_emit_string_is_empty_ios() {
        assert_eq!(emit_stdlib_call("string.is_empty", "ios"), ".isEmpty");
    }
    #[test]
    fn test_emit_string_length_android() {
        assert_eq!(emit_stdlib_call("string.length", "android"), ".length");
    }
    #[test]
    fn test_emit_string_length_ios() {
        assert_eq!(emit_stdlib_call("string.length", "ios"), ".count");
    }
    #[test]
    fn test_emit_string_to_int_android() {
        assert_eq!(emit_stdlib_call("string.to_int", "android"), ".toInt()!!");
    }
    #[test]
    fn test_emit_string_to_float_ios() {
        assert_eq!(emit_stdlib_call("string.to_float", "ios"), "Double(self)!");
    }
    #[test]
    fn test_emit_string_starts_with_android() {
        assert_eq!(emit_stdlib_call("string.starts_with", "android"), ".startsWith(x)");
    }
    #[test]
    fn test_emit_string_starts_with_ios() {
        assert_eq!(emit_stdlib_call("string.starts_with", "ios"), ".hasPrefix(x)");
    }
    #[test]
    fn test_emit_string_ends_with_android() {
        assert_eq!(emit_stdlib_call("string.ends_with", "android"), ".endsWith(x)");
    }
    #[test]
    fn test_emit_string_ends_with_ios() {
        assert_eq!(emit_stdlib_call("string.ends_with", "ios"), ".hasSuffix(x)");
    }

    // ── Number method mappings ────────────────────────────────────────────
    #[test]
    fn test_emit_number_abs_android() {
        assert_eq!(emit_stdlib_call("number.abs", "android"), "Math.abs(x)");
    }
    #[test]
    fn test_emit_number_abs_ios() {
        assert_eq!(emit_stdlib_call("number.abs", "ios"), "abs(x)");
    }
    #[test]
    fn test_emit_number_sqrt_android() {
        assert_eq!(emit_stdlib_call("number.sqrt", "android"), "Math.sqrt(x.toDouble())");
    }
    #[test]
    fn test_emit_number_sqrt_ios() {
        assert_eq!(emit_stdlib_call("number.sqrt", "ios"), "sqrt(x)");
    }
    #[test]
    fn test_emit_number_random_android() {
        assert_eq!(emit_stdlib_call("number.random", "android"), "Math.random()");
    }
    #[test]
    fn test_emit_number_random_ios() {
        assert_eq!(emit_stdlib_call("number.random", "ios"), "Double.random(in:0..<1)");
    }

    // ── List method mappings ──────────────────────────────────────────────
    #[test]
    fn test_emit_list_length_android() {
        assert_eq!(emit_stdlib_call("list.length", "android"), ".size");
    }
    #[test]
    fn test_emit_list_length_ios() {
        assert_eq!(emit_stdlib_call("list.length", "ios"), ".count");
    }
    #[test]
    fn test_emit_list_push_android() {
        assert_eq!(emit_stdlib_call("list.push", "android"), ".add(x)");
    }
    #[test]
    fn test_emit_list_push_ios() {
        assert_eq!(emit_stdlib_call("list.push", "ios"), ".append(x)");
    }

    // ── Invariant: filter then every mappings exist ───────────────────────
    #[test]
    fn test_filter_then_every_kotlin_mapping() {
        let filter = emit_stdlib_call("list.filter", "android");
        let every  = emit_stdlib_call("list.every", "android");
        assert!(!filter.is_empty());
        assert!(!every.is_empty());
    }

    // ── Invariant: map and length mappings exist ──────────────────────────
    #[test]
    fn test_map_length_mapping() {
        let map_k = emit_stdlib_call("list.map", "android");
        let map_s = emit_stdlib_call("list.map", "ios");
        assert!(!map_k.is_empty());
        assert!(!map_s.is_empty());
        let len_k = emit_stdlib_call("list.length", "android");
        let len_s = emit_stdlib_call("list.length", "ios");
        assert!(!len_k.is_empty());
        assert!(!len_s.is_empty());
    }

    // ── Invariant: reverse mapping exists for both platforms ──────────────
    #[test]
    fn test_reverse_mapping_both_platforms() {
        assert!(!emit_stdlib_call("list.reverse", "android").is_empty());
        assert!(!emit_stdlib_call("list.reverse", "ios").is_empty());
    }

    // ── Invariant: sum and average mappings exist ─────────────────────────
    #[test]
    fn test_sum_average_mapping() {
        assert!(!emit_stdlib_call("list.sum", "android").is_empty());
        assert!(!emit_stdlib_call("list.average", "android").is_empty());
        assert!(!emit_stdlib_call("list.sum", "ios").is_empty());
        assert!(!emit_stdlib_call("list.average", "ios").is_empty());
    }

    // ── Null safety operators ─────────────────────────────────────────────
    #[test]
    fn test_null_coalesce_android() {
        assert_eq!(emit_stdlib_call("null.coalesce", "android"), "?:");
    }
    #[test]
    fn test_null_coalesce_ios() {
        assert_eq!(emit_stdlib_call("null.coalesce", "ios"), "??");
    }
    #[test]
    fn test_safe_nav_both_platforms() {
        assert_eq!(emit_stdlib_call("null.safe_nav", "android"), "?.");
        assert_eq!(emit_stdlib_call("null.safe_nav", "ios"), "?.");
    }

    // ── Math namespace ────────────────────────────────────────────────────
    #[test]
    fn test_math_sqrt_android() {
        assert_eq!(emit_stdlib_call("math.sqrt", "android"), "Math.sqrt(x)");
    }
    #[test]
    fn test_math_pi_ios() {
        assert_eq!(emit_stdlib_call("math.pi", "ios"), "Double.pi");
    }
    #[test]
    fn test_math_pi_android() {
        assert_eq!(emit_stdlib_call("math.pi", "android"), "Math.PI");
    }
    #[test]
    fn test_math_sin_android() {
        assert_eq!(emit_stdlib_call("math.sin", "android"), "Math.sin(x)");
    }
    #[test]
    fn test_math_cos_ios() {
        assert_eq!(emit_stdlib_call("math.cos", "ios"), "cos(x)");
    }

    // ── Utility functions ─────────────────────────────────────────────────
    #[test]
    fn test_print_android() {
        assert_eq!(emit_stdlib_call("util.print", "android"), "println(x)");
    }
    #[test]
    fn test_print_ios() {
        assert_eq!(emit_stdlib_call("util.print", "ios"), "print(x)");
    }
    #[test]
    fn test_uuid_ios() {
        assert_eq!(emit_stdlib_call("util.uuid", "ios"), "UUID().uuidString");
    }
    #[test]
    fn test_uuid_android() {
        assert_eq!(emit_stdlib_call("util.uuid", "android"), "UUID.randomUUID().toString()");
    }

    // ── Date namespace ────────────────────────────────────────────────────
    #[test]
    fn test_date_now_android() {
        assert_eq!(emit_stdlib_call("date.now", "android"), "System.currentTimeMillis()");
    }
    #[test]
    fn test_date_now_ios() {
        assert_eq!(emit_stdlib_call("date.now", "ios"), "Date().timeIntervalSince1970");
    }

    // ── Object methods ────────────────────────────────────────────────────
    #[test]
    fn test_object_keys_android() {
        assert_eq!(emit_stdlib_call("object.keys", "android"), ".keys.toList()");
    }
    #[test]
    fn test_object_keys_ios() {
        assert_eq!(emit_stdlib_call("object.keys", "ios"), "Array(dict.keys)");
    }
    #[test]
    fn test_from_json_android() {
        assert_eq!(emit_stdlib_call("from_json", "android"), "Gson().fromJson(s, Map::class.java)");
    }

    // ── String interpolation ──────────────────────────────────────────────
    #[test]
    fn test_string_interpolation_replaces_vars() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("$name".to_string(), "World".to_string());
        vars.insert("$color".to_string(), "#FF0000".to_string());
        assert_eq!(resolve_string_interpolation("Hello $name", &vars), "Hello World");
        assert_eq!(resolve_string_interpolation("Color: $color", &vars), "Color: #FF0000");
    }
    #[test]
    fn test_string_interpolation_unknown_var_unchanged() {
        let vars = std::collections::HashMap::new();
        let s = "Hello $unknown";
        assert_eq!(resolve_string_interpolation(s, &vars), s);
    }
    #[test]
    fn test_string_interpolation_multiple_vars() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("$first".to_string(), "John".to_string());
        vars.insert("$last".to_string(), "Doe".to_string());
        assert_eq!(
            resolve_string_interpolation("$first $last", &vars),
            "John Doe"
        );
    }
    #[test]
    fn test_string_interpolation_empty_string() {
        let vars = std::collections::HashMap::new();
        assert_eq!(resolve_string_interpolation("", &vars), "");
    }

    // ── Unknown call returns empty string ─────────────────────────────────
    #[test]
    fn test_unknown_call_returns_empty() {
        assert_eq!(emit_stdlib_call("unknown.method", "android"), "");
        assert_eq!(emit_stdlib_call("unknown.method", "ios"), "");
    }
    #[test]
    fn test_empty_call_returns_empty() {
        assert_eq!(emit_stdlib_call("", "android"), "");
        assert_eq!(emit_stdlib_call("", "ios"), "");
    }

    // ── inject() doesn't crash on default (empty) AST ────────────────────
    #[test]
    fn test_inject_empty_ast() {
        let ast = AST::default();
        let result = inject(ast);
        assert_eq!(result.pages.len(), 0);
    }

    // ── inject() resolves vars in page children ───────────────────────────
    #[test]
    fn test_inject_resolves_string_vars_in_page() {
        use std::collections::HashMap;
        let mut ast = AST::default();
        ast.vars.insert("$greeting".to_string(), "Hello".to_string());

        let mut props: HashMap<String, Expr> = HashMap::new();
        props.insert("label".to_string(), Expr::Literal(Value::Str("$greeting World".to_string())));

        let node = ComponentNode {
            kind: "text".to_string(),
            props,
            ..Default::default()
        };

        let mut page = Page::default();
        page.children.push(node);
        ast.pages.push(page);

        let result = inject(ast);
        if let Expr::Literal(Value::Str(s)) = &result.pages[0].children[0].props["label"] {
            assert_eq!(s, "Hello World");
        } else {
            panic!("Expected string literal");
        }
    }

    // ── Call with args uses base_call matching ────────────────────────────
    #[test]
    fn test_emit_with_args_strips_for_matching() {
        // "string.contains(\"hello\")" should match "string.contains"
        let result = emit_stdlib_call("string.contains(\"hello\")", "android");
        assert_eq!(result, ".contains(x)");
    }
    #[test]
    fn test_emit_list_filter_with_args() {
        let result = emit_stdlib_call("list.filter(p)", "ios");
        assert_eq!(result, ".filter{p($0)}");
    }
}
