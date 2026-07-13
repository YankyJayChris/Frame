# Text & Content Components

Display text, icons, images, and interactive content elements.

---

## text

Display formatted text. Supports rich styling and string interpolation.

```fr
text: { content: "Hello, World!" }
text: {
    content: "Hello \(name), you have \(count) messages"
    styles: {
        font_size: 18sp
        font_weight: "bold"
        color: "#333333"
        text_overflow: ellipsis
        max_lines: 2
        font_family: "Helvetica"
    }
    on_click: handleTap()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | `""` | Text content |

| Style Props | Events | Children |
|-------------|--------|----------|
| `color`, `font_size`, `font_weight`, `font_family`, `text_overflow`, `max_lines`, `line_clamp`, `width`, `height`, `margin`, `padding`, `opacity` | `on_click` | No |

---

## button

Tappable button. Styled with layout properties.

```fr
button: { content: "Submit" }
button: {
    content: "Get Started"
    styles: {
        background: "#007BFF"
        color: "#FFFFFF"
        border_radius: 8dp
        padding: 12dp 24dp
        margin_top: 16dp
    }
    on_click: navigate("/home", clear_stack: true)
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | `""` | Button label |

| Style Props | Events | Children |
|-------------|--------|----------|
| All layout styles | `on_click` | No |

---

## icon

Displays an icon. Uses SF Symbols on iOS, Material Icons on Android, or custom SVG path data.

```fr
icon: { name: "heart"  styles: { color: "#FF0000"  width: 24dp  height: 24dp } }
icon: { path: "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5..."  styles: { width: 24  height: 24 } }
icon: {
    name: "gearshape"
    styles: { color: "#333"  width: 24  height: 24 }
    on_click: openSettings()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `name` | String | No | — | Platform icon name (SF Symbol or Material) |
| `path` | String | No | — | Custom SVG path data |

| Style Props | Events | Children |
|-------------|--------|----------|
| `color`, `font_weight`, `width`, `height`, `opacity`, `margin` | `on_click` | No |

---

## image

Displays an image from URL or local asset.

```fr
image: { src: "https://example.com/photo.jpg" }
image: {
    src: "https://example.com/photo.jpg"
    styles: { width: 100%  height: 200dp  fit: cover  border_radius: 12dp }
    on_click: enlarge()
}
image: { src: "assets/logo.png"  styles: { width: 120dp  height: 120dp  fit: contain } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Image URL or asset path |

| Style Props | Events | Children |
|-------------|--------|----------|
| `width`, `height`, `border_radius`, `opacity`, `margin`, `fit`, `clip_behavior` | `on_click` | No |

`fit` values: `cover`, `contain`, `fill`, `fit_width`, `fit_height`, `none`

---

## avatar

Circular avatar image. Automatically clips to circle.

```fr
avatar: { src: "https://i.pravatar.cc/80" }
avatar: { src: UserStore.user_avatar  styles: { width: 48dp  height: 48dp }  on_click: viewProfile() }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Avatar image URL |

| Style Props | Events | Children |
|-------------|--------|----------|
| `width`, `height`, `border_radius`, `opacity`, `margin`, `fit`, `clip_behavior` | `on_click` | No |

---

## badge

Notification badge showing a count.

```fr
badge: { count: 5 }
badge: { count: UserStore.unread_count }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `count` | Int | No | — | Badge number |

| Style Props | Events | Children |
|-------------|--------|----------|
| All layout styles | — | Yes (any) |

---

## chip

Compact interactive chip/tag element.

```fr
chip: { content: "React" }
chip: {
    content: "Filter"
    styles: { background: "#E3F2FD"  border_radius: 16dp  padding: 8dp 16dp }
    on_click: applyFilter()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Chip text |
| `label` | String | No | — | Optional label |

| Style Props | Events | Children |
|-------------|--------|----------|
| All layout styles | `on_click` | No |

---

## tag

Visual label/tag. Non-interactive (no click event).

```fr
tag: { content: "New" }
tag: { content: "Beta"  styles: { background: "#FFF3CD"  color: "#856404"  border_radius: 4dp  padding: 4dp 8dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Tag text |
| `label` | String | No | — | Optional label |

| Style Props | Events | Children |
|-------------|--------|----------|
| All layout styles | — | No |

---

## banner

Prominent banner with background color.

```fr
banner: {
    styles: { background: "#E3F2FD"  padding: 12dp  border_radius: 8dp }
    children: [
        text: { content: "New version available!"  styles: { font_weight: "bold" } }
    ]
    on_click: openUpdate()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Style Props | Events | Children |
|-------------|--------|----------|
| All layout styles | `on_click` | Yes (any) |

---

## skeleton

Loading placeholder that shows a gray shimmer animation.

```fr
skeleton: { show_if: UserStore.is_loading }
skeleton: { show_if: data == null  styles: { width: 100%  height: 80dp  border_radius: 8dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Style Props | Events |
|-------------|--------|
| All layout styles | — |

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

| Style Props | Events | Children |
|-------------|--------|----------|
| All layout styles | — | Yes (any) |
