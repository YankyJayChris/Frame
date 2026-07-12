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

/// Translate a dotted stdlib function call (e.g. `util.print`, `log.info`, `string.upper`)
/// into the platform-native equivalent, substituting already-emitted arguments.
///
/// Returns `None` if the function is not a known stdlib function.
pub fn translate_stdlib_call(func: &str, args: &[String], platform: &str) -> Option<String> {
    let is_android = platform == "android";
    match func {
        // ── String methods ────────────────────────────────────────────────
        "string.upper"       => Some(if is_android { format!("{}.uppercase()", args.first()?) } else { format!("{}.uppercased()", args.first()?) }),
        "string.lower"       => Some(if is_android { format!("{}.lowercase()", args.first()?) } else { format!("{}.lowercased()", args.first()?) }),
        "string.trim"        => Some(if is_android { format!("{}.trim()", args.first()?) } else { format!("{}.trimmingCharacters(in: .whitespaces)", args.first()?) }),
        "string.contains"    => Some(if is_android { format!("{}.contains({})", args.first()?, args.get(1)?) } else { format!("{}.contains({})", args.first()?, args.get(1)?) }),
        "string.starts_with" => Some(if is_android { format!("{}.startsWith({})", args.first()?, args.get(1)?) } else { format!("{}.hasPrefix({})", args.first()?, args.get(1)?) }),
        "string.ends_with"   => Some(if is_android { format!("{}.endsWith({})", args.first()?, args.get(1)?) } else { format!("{}.hasSuffix({})", args.first()?, args.get(1)?) }),
        "string.replace"     => Some(if is_android { format!("{}.replaceFirst({}, {})", args.first()?, args.get(1)?, args.get(2)?) } else { format!("{}.replacingOccurrences(of: {}, with: {})", args.first()?, args.get(1)?, args.get(2)?) }),
        "string.replace_all" => Some(if is_android { format!("{}.replace({}, {})", args.first()?, args.get(1)?, args.get(2)?) } else { format!("{}.replacingOccurrences(of: {}, with: {})", args.first()?, args.get(1)?, args.get(2)?) }),
        "string.split"       => Some(if is_android { format!("{}.split({})", args.first()?, args.get(1)?) } else { format!("{}.components(separatedBy: {})", args.first()?, args.get(1)?) }),
        "string.join"        => Some(if is_android { format!("{}.joinToString({})", args.first()?, args.get(1)?) } else { format!("{}.joined(separator: {})", args.first()?, args.get(1)?) }),
        "string.slice"       => Some(if is_android { format!("{}.substring({}, {})", args.first()?, args.get(1)?, args.get(2)?) } else { format!("String({}.prefix({})).dropFirst({})", args.first()?, args.get(2)?, args.get(1)?) }),
        "string.index_of"    => Some(if is_android { format!("{}.indexOf({})", args.first()?, args.get(1)?) } else { format!("{}.range(of: {})", args.first()?, args.get(1)?) }),
        "string.is_empty"    => Some(if is_android { format!("{}.isEmpty()", args.first()?) } else { format!("{}.isEmpty", args.first()?) }),
        "string.repeat"      => Some(if is_android { format!("{}.repeat({})", args.first()?, args.get(1)?) } else { format!("String(repeating: {}, count: {})", args.first()?, args.get(1)?) }),
        "string.pad_left"    => Some(if is_android { format!("{}.padStart({}, {})", args.first()?, args.get(1)?, args.get(2)?) } else { format!("padLeft({}, {}, {})", args.first()?, args.get(1)?, args.get(2)?) }),
        "string.pad_right"   => Some(if is_android { format!("{}.padEnd({}, {})", args.first()?, args.get(1)?, args.get(2)?) } else { format!("padRight({}, {}, {})", args.first()?, args.get(1)?, args.get(2)?) }),
        "string.to_int"      => Some(if is_android { format!("{}.toInt()!!", args.first()?) } else { format!("Int({})!", args.first()?) }),
        "string.to_float"    => Some(if is_android { format!("{}.toDouble()!!", args.first()?) } else { format!("Double({})!", args.first()?) }),
        "string.to_bool"     => Some(if is_android { format!("{}.toBooleanStrict()", args.first()?) } else { format!("Bool({})!", args.first()?) }),
        "string.length"      => Some(if is_android { format!("{}.length", args.first()?) } else { format!("{}.count", args.first()?) }),

        // ── Number functions ───────────────────────────────────────────────
        "number.abs"         => Some(if is_android { format!("Math.abs({})", args.first()?) } else { format!("abs({})", args.first()?) }),
        "number.sqrt"        => Some(if is_android { format!("Math.sqrt({}.toDouble())", args.first()?) } else { format!("sqrt({})", args.first()?) }),
        "number.floor"       => Some(if is_android { format!("kotlin.math.floor({})", args.first()?) } else { format!("floor({})", args.first()?) }),
        "number.ceil"        => Some(if is_android { format!("kotlin.math.ceil({})", args.first()?) } else { format!("ceil({})", args.first()?) }),
        "number.round"       => Some(if is_android { format!("kotlin.math.round({})", args.first()?) } else { format!("round({})", args.first()?) }),
        "number.min"         => Some(if is_android { format!("Math.min({}, {})", args.first()?, args.get(1)?) } else { format!("min({}, {})", args.first()?, args.get(1)?) }),
        "number.max"         => Some(if is_android { format!("Math.max({}, {})", args.first()?, args.get(1)?) } else { format!("max({}, {})", args.first()?, args.get(1)?) }),
        "number.random"      => Some(if is_android { "Math.random()".into() } else { "Double.random(in: 0..<1)".into() }),
        "number.clamp"       => Some(if is_android { format!("{}.coerceIn({}, {})", args.first()?, args.get(1)?, args.get(2)?) } else { format!("min(max({}, {}), {})", args.first()?, args.get(1)?, args.get(2)?) }),

        // ── List / collection methods ──────────────────────────────────────
        "list.length"        => Some(if is_android { format!("{}.size", args.first()?) } else { format!("{}.count", args.first()?) }),
        "list.push"          => Some(if is_android { format!("{}.add({})", args.first()?, args.get(1)?) } else { format!("{}.append({})", args.first()?, args.get(1)?) }),
        "list.pop"           => Some(if is_android { format!("{}.removeAt({}.size - 1)", args.first()?, args.first()?) } else { format!("{}.removeLast()", args.first()?) }),
        "list.remove_at"     => Some(if is_android { format!("{}.removeAt({})", args.first()?, args.get(1)?) } else { format!("{}.remove(at: {})", args.first()?, args.get(1)?) }),
        "list.contains"      => Some(if is_android { format!("{}.contains({})", args.first()?, args.get(1)?) } else { format!("{}.contains({})", args.first()?, args.get(1)?) }),
        "list.is_empty"      => Some(if is_android { format!("{}.isEmpty()", args.first()?) } else { format!("{}.isEmpty", args.first()?) }),
        "list.first"         => Some(if is_android { format!("{}.first()", args.first()?) } else { format!("{}.first!", args.first()?) }),
        "list.last"          => Some(if is_android { format!("{}.last()", args.first()?) } else { format!("{}.last!", args.first()?) }),
        "list.at"            => Some(if is_android { format!("{}[{}]", args.first()?, args.get(1)?) } else { format!("{}[{}]", args.first()?, args.get(1)?) }),
        "list.reverse"       => Some(if is_android { format!("{}.reversed()", args.first()?) } else { format!("{}.reversed()", args.first()?) }),
        "list.sort"          => Some(if is_android { format!("{}.sorted()", args.first()?) } else { format!("{}.sorted()", args.first()?) }),
        "list.sum"           => Some(if is_android { format!("{}.sum()", args.first()?) } else { format!("{}.reduce(0, +)", args.first()?) }),
        "list.average"       => Some(if is_android { format!("{}.average()", args.first()?) } else { format!("Double({}.reduce(0, +)) / Double({}.count)", args.first()?, args.first()?) }),

        // ── Math functions ─────────────────────────────────────────────────
        "math.abs"           => Some(if is_android { format!("Math.abs({})", args.first()?) } else { format!("abs({})", args.first()?) }),
        "math.sqrt"          => Some(if is_android { format!("Math.sqrt({})", args.first()?) } else { format!("sqrt({})", args.first()?) }),
        "math.sin"           => Some(if is_android { format!("Math.sin({})", args.first()?) } else { format!("sin({})", args.first()?) }),
        "math.cos"           => Some(if is_android { format!("Math.cos({})", args.first()?) } else { format!("cos({})", args.first()?) }),
        "math.tan"           => Some(if is_android { format!("Math.tan({})", args.first()?) } else { format!("tan({})", args.first()?) }),
        "math.pi"            => Some(if is_android { "Math.PI".into() } else { "Double.pi".into() }),
        "math.e"             => Some(if is_android { "Math.E".into() } else { "M_E".into() }),
        "math.pow"           => Some(if is_android { format!("Math.pow({}, {})", args.first()?, args.get(1)?) } else { format!("pow({}, {})", args.first()?, args.get(1)?) }),
        "math.log"           => Some(if is_android { format!("Math.log({})", args.first()?) } else { format!("log({})", args.first()?) }),
        "math.log10"         => Some(if is_android { format!("Math.log10({})", args.first()?) } else { format!("log10({})", args.first()?) }),

        // ── Date/time functions ────────────────────────────────────────────
        "date.now"           => Some(if is_android { "System.currentTimeMillis()".into() } else { "Date().timeIntervalSince1970".into() }),
        "date.from_timestamp" => Some(if is_android { format!("Date({})", args.first()?) } else { format!("Date(timeIntervalSince1970: {})", args.first()?) }),
        "date.format"        => Some(if is_android { format!("SimpleDateFormat({}).format(this)", args.first()?) } else { format!("DateFormatter().string(from: {})", args.first()?) }),
        "date.diff"          => Some(if is_android { format!("ChronoUnit.DAYS.between(this, {})", args.first()?) } else { format!("Calendar.current.dateComponents([.day], from: this, to: {}).day!", args.first()?) }),

        // ── Utility functions ──────────────────────────────────────────────
        "util.type_of"       => Some(if is_android { format!("{}::class.simpleName", args.first()?) } else { format!("type(of: {})", args.first()?) }),
        "util.is_null"       => Some(if is_android { format!("{} == null", args.first()?) } else { format!("{} == nil", args.first()?) }),
        "util.is_not_null"   => Some(if is_android { format!("{} != null", args.first()?) } else { format!("{} != nil", args.first()?) }),
        "util.print"         => Some(if is_android { format!("println({})", args.first()?) } else { format!("print({})", args.first()?) }),
        "util.assert"        => Some(if is_android { format!("assert({})", args.first()?) } else { format!("assert({})", args.first()?) }),
        "util.uuid"          => Some(if is_android { "UUID.randomUUID().toString()".into() } else { "UUID().uuidString".into() }),
        "util.hash"          => Some(if is_android { format!("{}.hashCode().toString()", args.first()?) } else { format!("String({}.hashValue)", args.first()?) }),
        "util.encode_base64" => Some(if is_android { format!("Base64.getEncoder().encodeToString({}.toByteArray())", args.first()?) } else { format!("Data({}.utf8).base64EncodedString()", args.first()?) }),
        "util.decode_base64" => Some(if is_android { format!("String(Base64.getDecoder().decode({}))", args.first()?) } else { format!("String(data: Data(base64Encoded: {})!, encoding: .utf8)!", args.first()?) }),
        "util.encode_url"    => Some(if is_android { format!("URLEncoder.encode({}, \"UTF-8\")", args.first()?) } else { format!("{}.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed)!", args.first()?) }),
        "util.decode_url"    => Some(if is_android { format!("URLDecoder.decode({}, \"UTF-8\")", args.first()?) } else { format!("{}.removingPercentEncoding!", args.first()?) }),

        // ── Object / Map functions ─────────────────────────────────────────
        "object.keys"        => Some(if is_android { format!("{}.keys.toList()", args.first()?) } else { format!("Array({}.keys)", args.first()?) }),
        "object.values"      => Some(if is_android { format!("{}.values.toList()", args.first()?) } else { format!("Array({}.values)", args.first()?) }),
        "object.has_key"     => Some(if is_android { format!("{}.containsKey({})", args.first()?, args.get(1)?) } else { format!("{}.keys.contains({})", args.first()?, args.get(1)?) }),

        // ── JSON ───────────────────────────────────────────────────────────
        "from_json"          => Some(if is_android { format!("Gson().fromJson({}, Map::class.java)", args.first()?) } else { format!("try! JSONSerialization.jsonObject(with: {}.data(using: .utf8)!) as! [String: Any]", args.first()?) }),
        "to_json"            => Some(if is_android { format!("Gson().toJson({})", args.first()?) } else { format!("String(data: try! JSONSerialization.data(withJSONObject: {}), encoding: .utf8)!", args.first()?) }),

        // ── Logging ────────────────────────────────────────────────────────
        "log.info"           => Some(if is_android {
            format!("android.util.Log.i(\"Frame\", {})", args.first()?)
        } else {
            format!("os_log(.info, \"{{}}\", {})", args.first()?)
        }),
        "log.warn"           => Some(if is_android {
            format!("android.util.Log.w(\"Frame\", {})", args.first()?)
        } else {
            format!("os_log(.default, \"{{}}\", {})", args.first()?)
        }),
        "log.error"          => Some(if is_android {
            format!("android.util.Log.e(\"Frame\", {})", args.first()?)
        } else {
            format!("os_log(.error, \"{{}}\", {})", args.first()?)
        }),
        "log.debug"          => Some(if is_android {
            format!("android.util.Log.d(\"Frame\", {})", args.first()?)
        } else {
            format!("os_log(.debug, \"{{}}\", {})", args.first()?)
        }),
        "log.verbose"        => Some(if is_android {
            format!("android.util.Log.v(\"Frame\", {})", args.first()?)
        } else {
            format!("os_log(.debug, \"{{}}\", {})", args.first()?)
        }),

        // Unknown
        _ => None,
    }
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

        // ── Logging ────────────────────────────────────────────────────────
        "log.info"    => if is_android { "android.util.Log.i(\"Frame\", x)".into() } else { "os_log(.info, \"{x}\")".into() },
        "log.warn"    => if is_android { "android.util.Log.w(\"Frame\", x)".into() } else { "os_log(.default, \"{x}\")".into() },
        "log.error"   => if is_android { "android.util.Log.e(\"Frame\", x)".into() } else { "os_log(.error, \"{x}\")".into() },
        "log.debug"   => if is_android { "android.util.Log.d(\"Frame\", x)".into() } else { "os_log(.debug, \"{x}\")".into() },
        "log.verbose" => if is_android { "android.util.Log.v(\"Frame\", x)".into() } else { "os_log(.debug, \"{x}\")".into() },

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
        // Try both $key and bare key patterns for backward compatibility
        let dollar_key = format!("${}", k);
        result = result.replace(dollar_key.as_str(), v.as_str());
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
        vars.insert("name".to_string(), "World".to_string());
        vars.insert("color".to_string(), "#FF0000".to_string());
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
        vars.insert("first".to_string(), "John".to_string());
        vars.insert("last".to_string(), "Doe".to_string());
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
        ast.vars.insert("greeting".to_string(), "Hello".to_string());

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

    // ── Logging functions (translate_stdlib_call) ─────────────────────────
    #[test]
    fn test_log_info_android() {
        let result = translate_stdlib_call("log.info", &["\"hello\"".to_string()], "android");
        assert_eq!(result, Some("android.util.Log.i(\"Frame\", \"hello\")".into()));
    }
    #[test]
    fn test_log_info_ios() {
        let result = translate_stdlib_call("log.info", &["\"hello\"".to_string()], "ios");
        assert_eq!(result, Some("os_log(.info, \"{}\", \"hello\")".into()));
    }
    #[test]
    fn test_log_warn_android() {
        let result = translate_stdlib_call("log.warn", &["\"warning\"".to_string()], "android");
        assert_eq!(result, Some("android.util.Log.w(\"Frame\", \"warning\")".into()));
    }
    #[test]
    fn test_log_warn_ios() {
        let result = translate_stdlib_call("log.warn", &["\"warning\"".to_string()], "ios");
        assert_eq!(result, Some("os_log(.default, \"{}\", \"warning\")".into()));
    }
    #[test]
    fn test_log_error_android() {
        let result = translate_stdlib_call("log.error", &["\"error\"".to_string()], "android");
        assert_eq!(result, Some("android.util.Log.e(\"Frame\", \"error\")".into()));
    }
    #[test]
    fn test_log_error_ios() {
        let result = translate_stdlib_call("log.error", &["\"error\"".to_string()], "ios");
        assert_eq!(result, Some("os_log(.error, \"{}\", \"error\")".into()));
    }
    #[test]
    fn test_util_print_translate_android() {
        let result = translate_stdlib_call("util.print", &["\"hello\"".to_string()], "android");
        assert_eq!(result, Some("println(\"hello\")".into()));
    }
    #[test]
    fn test_util_print_translate_ios() {
        let result = translate_stdlib_call("util.print", &["\"hello\"".to_string()], "ios");
        assert_eq!(result, Some("print(\"hello\")".into()));
    }
    #[test]
    fn test_string_upper_translate_android() {
        let result = translate_stdlib_call("string.upper", &["myStr".to_string()], "android");
        assert_eq!(result, Some("myStr.uppercase()".into()));
    }
    #[test]
    fn test_string_upper_translate_ios() {
        let result = translate_stdlib_call("string.upper", &["myStr".to_string()], "ios");
        assert_eq!(result, Some("myStr.uppercased()".into()));
    }
    #[test]
    fn test_unknown_translate_returns_none() {
        let result = translate_stdlib_call("foo.bar", &["x".to_string()], "android");
        assert_eq!(result, None);
    }
    #[test]
    fn test_log_debug_android() {
        let result = translate_stdlib_call("log.debug", &["\"dbg\"".to_string()], "android");
        assert_eq!(result, Some("android.util.Log.d(\"Frame\", \"dbg\")".into()));
    }
    #[test]
    fn test_log_verbose_android() {
        let result = translate_stdlib_call("log.verbose", &["\"v\"".to_string()], "android");
        assert_eq!(result, Some("android.util.Log.v(\"Frame\", \"v\")".into()));
    }
}
