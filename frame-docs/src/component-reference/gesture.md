# Gesture Components

Gesture components wrap other components to handle touch gestures like swiping, dragging, pulling to refresh, and long-pressing. All accept the full set of layout styles.

---

## swipeable

Wraps content with swipe gesture detection.

```fr
swipeable: {
    on_swipe: handleSwipe()
    children: [
        card: { children: [ text: { content: "Swipe me!" } ] }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_swipe` | Yes (any) |

**Swipe to reveal actions:**

```fr
swipeable: {
    on_swipe: handleSwipeAction()
    children: [
        row: {
            children: [
                button: { content: "Delete"  styles: { background: "#FF0000" }  on_click: deleteItem() }
                button: { content: "Archive"  on_click: archiveItem() }
            ]
        }
    ]
}
```

The swipe direction is detected automatically. The `on_swipe` handler receives the swipe direction as context.

---

## draggable

Makes children draggable with pan gesture.

```fr
draggable: {
    on_drag: handleDrag()
    children: [
        text: { content: "Drag me around" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_drag` | Yes (any) |

The `on_drag` handler receives the current drag position and offset.

---

## refresh

Pull-to-refresh container.

```fr
refresh: {
    refreshing: UserStore.is_refreshing
    on_refresh: wait:UserStore.refresh()
    children: [
        list: {
            data: UserStore.items
            build: (item) => { text: { content: item.name } }
        }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `refreshing` | Bool | No | — | Whether refreshing is active |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_refresh` | Yes (any) |

---

## long_press

Detects long-press gesture on wrapped content.

```fr
long_press: {
    on_long_press: handleLongPress()
    children: [
        card: { children: [ text: { content: "Press and hold" } ] }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | Accepts children |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_long_press` | Yes (any) |
