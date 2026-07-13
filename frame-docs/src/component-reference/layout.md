# Layout Components

Layout components define the structure and arrangement of your UI. All layout components accept the full set of layout styles (`width`, `height`, `padding`, `margin`, `background`, `border_radius`, `border`, `opacity`, `overflow`, `flex`, `direction`, `align`, `justify`, `gap`, `min_width`, `max_width`, `min_height`, `max_height`).

---

## scaffold

Top-level screen structure. Handles safe area, app bar, and bottom navigation slots. Designed to wrap page content.

```fr
scaffold: {
    styles: { safe_area: true }
    children: [
        app_bar: { title: "My App"  leading: "line.3.horizontal" }
        column: {
            styles: { padding: 16dp }
            children: [ text: { content: "Content here" } ]
        }
        bottom_navigation_bar: {
            children: [
                tab: { content: "Home" }
                tab: { content: "Profile" }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Routes first `app_bar` → top bar slot, first `bottom_navigation_bar` → bottom bar slot, remaining → body |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |

---

## column

Vertically arranges children in a flex column. The most common layout component.

```fr
column: {
    styles: {
        width: 100%
        height: 100%
        padding: 16dp
        gap: 12dp
        overflow: scroll
    }
    children: [
        text: { content: "Title"  styles: { font_size: 18sp  font_weight: "bold" } }
        text: { content: "Body content"  styles: { font_size: 14sp  color: "#666" } }
        button: { content: "Action"  on_click: handleAction() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click`, `on_scroll`, `on_scroll_end` | Yes (any) |

| Platform | Mapping |
|----------|---------|
| iOS | `UIStackView` axis = `.vertical` |
| Android | `Column` composable |

---

## row

Horizontally arranges children in a flex row.

```fr
row: {
    styles: { gap: 8dp  padding: 16dp  justify: "space_between"  align: "center" }
    children: [
        text: { content: "Left" }
        text: { content: "Center" }
        text: { content: "Right" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click`, `on_scroll`, `on_scroll_end` | Yes (any) |

| Platform | Mapping |
|----------|---------|
| iOS | `UIStackView` axis = `.horizontal` |
| Android | `Row` composable |

---

## container

Generic box container — a `UIView` / `Box` with no layout direction.

```fr
container: {
    styles: {
        background: "#F5F5F5"
        border_radius: 12dp
        padding: 16dp
        overflow: hidden
        width: 200
        height: 200
    }
    children: [
        text: { content: "Inside box" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click` | Yes (any) |

---

## stack

Positions children in z-order (layered on top of each other). Supports `alignment` for child positioning.

```fr
stack: {
    alignment: center
    children: [
        image: { src: "https://example.com/bg.jpg"  styles: { width: 100%  height: 200dp } }
        text: { content: "Overlay"  styles: { color: "#FFFFFF"  font_size: 18sp } }
    ]
}
```

**Positioned children:**

```fr
stack: {
    styles: { width: 300  height: 300 }
    children: [
        text: { content: "Centered" }
        text: {
            content: "Top Left"
            positioned: { top: 8  left: 8 }
        }
        text: {
            content: "Bottom Right"
            positioned: { bottom: 8  right: 8  width: 120  height: 40 }
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `alignment` | String | No | — | `top_left`, `top_center`, `top_right`, `center_left`, `center`, `center_right`, `bottom_left`, `bottom_center`, `bottom_right` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click` | Yes (any, can use `positioned:{}`) |

---

## card

Elevated container with platform shadow.

```fr
card: {
    styles: { padding: 16dp  margin_top: 8dp  border_radius: 12dp }
    children: [
        text: { content: "Card Title"  styles: { font_size: 18sp  font_weight: "bold" } }
        spacer: { styles: { height: 8dp } }
        text: { content: "Card body text."  styles: { font_size: 14sp  color: "#666" } }
        button: { content: "Learn More"  on_click: openDetail() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click` | Yes (any) |

---

## divider

Thin horizontal separator line.

```fr
divider: {}
divider: { styles: { color: "#E0E0E0"  margin: 8dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| `color`, `margin` | — |

---

## spacer

Invisible space used for flexible gaps.

```fr
spacer: { styles: { height: 16dp } }
spacer: { styles: { width: 8dp } }
spacer: { styles: { width: 100%  height: 20dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| `width`, `height` | — |

---

## scroll_view

Scrollable wrapper. Contents can overflow and scroll.

```fr
scroll_view: {
    styles: { width: 100%  height: 100% }
    children: [
        column: {
            styles: { gap: 12dp  padding: 16dp }
            children: [
                text: { content: "Item 1" }
                text: { content: "Item 2" }
                text: { content: "Item 3 (scroll to see)" }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_scroll`, `on_scroll_end` | Yes (any) |

---

## grid

Grid layout with fixed columns.

```fr
grid: {
    columns: 2
    styles: { gap: 8dp  padding: 16dp }
    children: [
        card: { children: [ text: { content: "Item 1" } ] }
        card: { children: [ text: { content: "Item 2" } ] }
        card: { children: [ text: { content: "Item 3" } ] }
        card: { children: [ text: { content: "Item 4" } ] }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `columns` | Int | No | — | Number of columns |
| `data` | Expression | No | — | Data source (alternative to children) |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any). Falls back to children if no `data` prop |

---

## list

Data-bound repeating list. Provide `data` (expression) and `build` (item render function).

```fr
list: {
    data: UserStore.items
    build: (item) => {
        text: { content: item.name  styles: { padding: 8dp } }
    }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `data` | Expression | No | — | List data source |
| `build` | Expression | No | — | Render function `(item) => { ... }` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_scroll`, `on_scroll_end` | Yes (fallback when no data/build) |

---

## form

Form container with validation schema.

```fr
form: {
    schema: UserSchema
    children: [
        input: { value: formName  placeholder: "Name" }
        input: { value: formEmail  placeholder: "Email" }
        button: { content: "Submit"  on_click: submitForm() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `schema` | String | No | — | `:validation` block name to apply |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_submit` | Yes (any) |

---

## accordion

Expandable/collapsible section with a title header.

```fr
accordion: {
    title: "More Details"
    styles: { padding: 12dp  border_radius: 8dp }
    children: [
        text: { content: "Hidden content revealed on tap"  styles: { padding: 8dp } }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | String | No | — | Header text |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |

---

## timeline

Vertical timeline display. Each child becomes a timeline item.

```fr
timeline: {
    styles: { padding: 16dp }
    children: [
        text: { content: "Step 1: Created"  styles: { font_size: 14sp } }
        text: { content: "Step 2: In Review"  styles: { font_size: 14sp } }
        text: { content: "Step 3: Approved"  styles: { font_size: 14sp } }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |

---

## item

Generic list item wrapper. Used inside `list` or `grid` children.

```fr
item: {
    styles: { padding: 12dp  border_radius: 8dp  background: "#FFF" }
    children: [
        text: { content: "List item" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |

---

## skeleton

Loading placeholder that shows a gray shimmer animation.

```fr
skeleton: { show_if: UserStore.is_loading }
skeleton: {
    show_if: data == null
    styles: { width: 100%  height: 80dp  border_radius: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| All layout | — |

---

## table

Tabular data display.

```fr
table: {
    data: UserStore.rows
    styles: { width: 100%  padding: 8dp }
    children: [
        row: {
            children: [
                text: { content: "Name"  styles: { font_weight: "bold" } }
                text: { content: "Email"  styles: { font_weight: "bold" } }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `data` | Expression | No | — | Table data source |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |
