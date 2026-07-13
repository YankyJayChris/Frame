# Standard Library

Frame's standard library provides built-in functions for string manipulation, list operations, math, dates, JSON, and utilities.

## String Methods

```fr
string.upper("hello")              // "HELLO"
string.lower("HELLO")              // "hello"
string.trim("  hi  ")              // "hi"
string.contains("hello", "el")     // true
string.starts_with("hello", "he")  // true
string.ends_with("hello", "lo")    // true
string.replace("a-b-c", "-", "_")  // "a_b_c"
string.replace_all("a-b-c", "-", "_") // "a_b_c"
string.split("a,b,c", ",")         // ["a", "b", "c"]
string.join(["a", "b", "c"], ",")  // "a,b,c"
string.length("hello")             // 5
string.is_empty("")                // true
string.slice("hello", 1, 4)        // "ell"
string.to_int("42")                // 42
string.to_float("3.14")            // 3.14
string.pad_left("5", 3, "0")       // "005"
string.pad_right("5", 3, "0")      // "500"
```

### String Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `upper` | `(s: string) -> string` | Convert to uppercase |
| `lower` | `(s: string) -> string` | Convert to lowercase |
| `trim` | `(s: string) -> string` | Remove leading/trailing whitespace |
| `split` | `(s: string, sep: string) -> list` | Split by separator |
| `replace` | `(s: string, from: string, to: string) -> string` | Replace first match |
| `replace_all` | `(s: string, from: string, to: string) -> string` | Replace all matches |
| `contains` | `(s: string, substr: string) -> bool` | Check if substring exists |
| `starts_with` | `(s: string, prefix: string) -> bool` | Check prefix |
| `ends_with` | `(s: string, suffix: string) -> bool` | Check suffix |
| `substring` | `(s: string, start: int, end: int) -> string` | Extract substring |
| `length` | `(s: string) -> int` | Get string length |
| `is_empty` | `(s: string) -> bool` | Check if empty |
| `join` | `(items: list, sep: string) -> string` | Join list into string |
| `to_int` | `(s: string) -> int` | Parse to integer |
| `to_float` | `(s: string) -> float` | Parse to float |
| `pad_left` | `(s: string, len: int, pad: string) -> string` | Left pad |
| `pad_right` | `(s: string, len: int, pad: string) -> string` | Right pad |

## List Methods

```fr
list.length([1, 2, 3])             // 3
list.contains([1, 2, 3], 2)        // true
list.is_empty([])                  // true
list.first([1, 2, 3])              // 1
list.last([1, 2, 3])               // 3
list.reverse([1, 2, 3])            // [3, 2, 1]
list.sum([1, 2, 3])                // 6
list.average([1, 2, 3])            // 2.0
```

### List Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `length` | `(list: list) -> int` | Number of elements |
| `map` | `(list: list, fn: lambda) -> list` | Transform each element |
| `filter` | `(list: list, fn: lambda) -> list` | Filter elements |
| `reduce` | `(list: list, fn: lambda, initial: any) -> any` | Reduce to single value |
| `sort` | `(list: list) -> list` | Sort elements |
| `find` | `(list: list, fn: lambda) -> any` | Find first match |
| `push` | `(list: list, item: any) -> list` | Append element |
| `pop` | `(list: list) -> any` | Remove and return last element |
| `remove_at` | `(list: list, index: int) -> list` | Remove at index |
| `contains` | `(list: list, item: any) -> bool` | Check if contains |
| `is_empty` | `(list: list) -> bool` | Check if empty |
| `join` | `(list: list, sep: string) -> string` | Join to string |
| `first` | `(list: list) -> any` | First element |
| `last` | `(list: list) -> any` | Last element |
| `reverse` | `(list: list) -> list` | Reversed copy |
| `sum` | `(list: list) -> int/float` | Sum of numeric elements |
| `average` | `(list: list) -> float` | Average of numeric elements |

## Math Functions

```fr
number.abs(-5)                      // 5
number.sqrt(16)                     // 4
number.floor(3.7)                   // 3
number.ceil(3.2)                    // 4
number.round(3.5)                   // 4
number.min(3, 7)                    // 3
number.max(3, 7)                    // 7
number.clamp(15, 0, 10)             // 10
number.random()                     // 0.0..1.0
```

### Math Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs` | `(x: int/float) -> int/float` | Absolute value |
| `min` | `(a: int/float, b: int/float) -> int/float` | Minimum |
| `max` | `(a: int/float, b: int/float) -> int/float` | Maximum |
| `round` | `(x: float) -> int` | Round to nearest integer |
| `floor` | `(x: float) -> int` | Round down |
| `ceil` | `(x: float) -> int` | Round up |
| `sqrt` | `(x: float) -> float` | Square root |
| `pow` | `(base: float, exp: float) -> float` | Power |
| `random` | `() -> float` | Random number 0.0–1.0 |
| `clamp` | `(val: int/float, min: int/float, max: int/float) -> int/float` | Clamp value |

## Date Functions

```fr
date.now()                           // current timestamp
date.format(date.now(), "yyyy-MM-dd") // "2026-07-13"
date.parse("2026-07-13", "yyyy-MM-dd") // timestamp
date.add_days(date.now(), 7)         // 7 days from now
date.add_months(date.now(), 1)       // 1 month from now
date.difference(date1, date2)        // difference in ms
```

### Date Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `now` | `() -> timestamp` | Current date/time |
| `format` | `(ts: timestamp, fmt: string) -> string` | Format timestamp |
| `parse` | `(s: string, fmt: string) -> timestamp` | Parse date string |
| `add_days` | `(ts: timestamp, days: int) -> timestamp` | Add days |
| `add_months` | `(ts: timestamp, months: int) -> timestamp` | Add months |
| `difference` | `(a: timestamp, b: timestamp) -> int` | Difference in ms |

## JSON Functions

```fr
from_json('{"name":"Alice"}')        // { name: "Alice" }
to_json({name: "Alice"})             // '{"name":"Alice"}'
```

### JSON Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `from_json` | `(s: string) -> object` | Parse JSON string |
| `to_json` | `(obj: object) -> string` | Serialize to JSON string |

## Object/Map Functions

```fr
object.keys({a: 1, b: 2})           // ["a", "b"]
object.values({a: 1, b: 2})         // [1, 2]
object.has_key({a: 1}, "a")         // true
```

## Utility Functions

```fr
util.print("hello")                  // prints to console
util.type_of(42)                     // "int"
util.is_null(null)                   // true
util.is_not_null("hello")            // true
util.uuid()                          // "550e8400-e29b-41d4-a716-446655440000"
util.encode_base64("hello")          // "aGVsbG8="
util.decode_base64("aGVsbG8=")       // "hello"
util.encode_url("hello world")       // "hello%20world"
util.decode_url("hello%20world")     // "hello world"
```

### Utility Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `print` | `(msg: any) -> void` | Print to console |
| `uuid` | `() -> string` | Generate UUID v4 |
| `type_of` | `(val: any) -> string` | Get type name |
| `is_null` | `(val: any) -> bool` | Check if null |
| `is_not_null` | `(val: any) -> bool` | Check if not null |
| `encode_base64` | `(s: string) -> string` | Base64 encode |
| `decode_base64` | `(s: string) -> string` | Base64 decode |
| `encode_url` | `(s: string) -> string` | URL encode |
| `decode_url` | `(s: string) -> string` | URL decode |
