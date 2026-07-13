# Feedback Components

Feedback components provide transient notifications, progress indicators, and contextual tooltips. All accept the full set of layout styles.

---

## toast

Transient notification that auto-dismisses.

```fr
toast: { message: "Saved successfully!" }
toast: { message: "Error saving data"  duration: 3000 }
button: { content: "Show Toast"  on_click: showToast("Hello from Frame!") }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `message` | String | No | — | Toast message text |
| `duration` | Int | No | — | Display duration in ms |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | No |

---

## tooltip

Contextual tooltip on hover/long-press.

```fr
tooltip: {
    text: "This is helpful info"
    styles: { padding: 8dp }
    children: [
        text: { content: "Hover over me" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `text` | String | No | — | Tooltip text |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | Yes (any — the trigger element) |

---

## progress_bar

Horizontal progress bar. Value from 0.0 to 1.0.

```fr
progress_bar: { value: 0.65 }
progress_bar: { value: UserStore.upload_progress  styles: { width: 100%  height: 4dp  border_radius: 2dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Float | No | — | Progress 0.0–1.0 |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | No |

---

## progress_circle

Circular progress indicator.

```fr
progress_circle: { value: 0.8 }
progress_circle: { value: UserStore.loading_progress }
progress_circle: { styles: { width: 48dp  height: 48dp  color: "#007BFF" } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Float | No | — | Progress 0.0–1.0 |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | No |
