# Navigation Components

Navigation components handle screen structure, tab navigation, dialogs, and menus. All accept the full set of layout styles.

---

## app_bar

Top app bar with title, leading icon, and trailing action items.

```fr
app_bar: { title: "Home" }
app_bar: {
    title: "Frame App"
    leading: "line.3.horizontal"
    children: [
        icon: { name: "magnifyingglass"  on_click: openSearch() }
        icon: { name: "gearshape"  on_click: openSettings() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | String | No | — | Title text |
| `leading` | String | No | — | Icon name for nav icon (hamburger, back) |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (icon/button children become trailing actions) |

**Usage:** Place as direct child of `scaffold`. On Android → `TopAppBar` slot. On iOS → `navigationItem.title` + bar button items.

---

## bottom_navigation_bar

Bottom tab navigation bar. Designed as a scaffold child.

```fr
bottom_navigation_bar: {
    styles: { background: "#FFFFFF" }
    children: [
        tab: { content: "Home"  icon: "house.fill" }
        tab: { content: "Search"  icon: "magnifyingglass" }
        tab: { content: "Profile"  icon: "person.fill" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Children rendered as tab items |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |

---

## sidebar

Side drawer panel. Pins to left or right edge.

```fr
sidebar: {
    side: "left"
    width: "280"
    styles: { background: "#F8F9FA"  padding: 8dp }
    children: [
        text: { content: "Menu"  styles: { font_weight: "bold"  padding: 8dp } }
        button: { content: "Dashboard"  on_click: navigate("/dashboard") }
        button: { content: "Profile"  on_click: navigate("/profile/1") }
        button: { content: "Settings"  on_click: navigate_modal("/settings") }
        divider: {}
        chip: { content: "Important" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `side` | String | No | `"left"` | `"left"` or `"right"` |
| `width` | String | No | `"260"` | Width in dp |
| `collapsed` | Bool | No | — | Whether collapsed |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any) |

---

## floating_action_button

Circular action button (FAB). Positioned at bottom-end by default.

```fr
floating_action_button: { icon: "plus"  on_click: handleAdd() }
floating_action_button: {
    content: "Save"
    icon: "checkmark"
    position: "bottom_end"
    on_click: saveData()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Label text |
| `icon` | String | No | — | Icon name |
| `position` | String | No | `"bottom_end"` | `bottom_end`, `bottom_start`, `top_end`, `top_start` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click` | Yes (rendered inside FAB) |

---

## tab_bar

Horizontal tab bar. Only accepts `tab` children.

```fr
tab_bar: {
    selected: 0
    children: [
        tab: { content: "Chats"  icon: "message.fill"  on_select: switchToChats() }
        tab: { content: "Status"  icon: "circle.fill"  on_select: switchToStatus() }
        tab: { content: "Calls"  icon: "phone.fill"  on_select: switchToCalls() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `selected` | Int | No | — | Selected tab index |
| `current` | Int | No | — | Synonym for `selected` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (only `tab` children) |

---

## tab

Individual tab item. Used inside `tab_bar`.

```fr
tab: { content: "Home"  icon: "house.fill"  on_click: goHome() }
tab: { content: "Settings"  icon: "gearshape"  on_select: openSettings() }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | String | No | — | Tab label |
| `icon` | String | No | — | Tab icon name |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click`, `on_select` | No |

---

## bottom_sheet

Modal bottom sheet. Slides up from bottom.

```fr
bottom_sheet: {
    styles: { padding: 16dp }
    children: [
        text: { content: "Sheet Title"  styles: { font_size: 18sp  font_weight: "bold" } }
        text: { content: "Sheet content" }
        button: { content: "Close"  on_click: navigate_dismiss() }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_unmount` | Yes (any) |

---

## modal

Alert-style dialog. For simple confirmations and messages.

```fr
modal: { title: "Confirm"  message: "Are you sure?" }
modal: {
    title: "Delete Item"
    message: "This action cannot be undone."
    on_unmount: handleDismiss()
    children: [
        row: {
            styles: { gap: 8dp  justify: "end" }
            children: [
                button: { content: "Cancel"  on_click: navigate_dismiss() }
                button: { content: "Delete"  styles: { color: "#FF0000" }  on_click: confirmDelete() }
            ]
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | String | No | — | Dialog title |
| `message` | String | No | — | Dialog message |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_unmount` | Yes (any) |

---

## Additional Navigation Components

The following navigation components are planned and will be available in a future release:

| Component | Description |
|-----------|-------------|
| `navigation_rail` | Vertical navigation rail (typically on tablets) |
| `drawer` | Side drawer menu that slides in from the edge |
| `popup_menu` | Context/popup menu anchored to a trigger element |
