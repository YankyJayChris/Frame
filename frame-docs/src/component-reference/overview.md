# Component Reference Overview

Frame ships with **64 built-in components** organized into 7 categories: Layout, Text/Content, Input, Navigation, Feedback, Media, and Gesture. Every built-in component name is **lowercase** — these are grammar-level keywords in the `.fr` language. PascalCase names like `UserCard: { }` are always user-defined components.

## Component Anatomy

Every component has the same general structure:

```fr
component_name: {
    // Required and optional props
    prop_name: value

    // Style block
    styles: {
        width: 100%
        padding: 16dp
        background: "#F5F5F5"
    }

    // Events
    on_click: handlerName()

    // Children (if allowed)
    children: [
        child_component: { }
    ]
}
```

## Props

Props are named arguments passed to a component. They have types (`string`, `int`, `float`, `bool`, `expr`) and can be required (no default — must be provided) or optional (with or without a default value).

```fr
// Required prop
image: { src: "https://example.com/photo.jpg" }

// Optional prop with default
otp_input: { length: 6 }

// Optional prop without default
button: { content: "Submit" }
```

## Children

Components that accept children use a `children: [ ... ]` list. Some components restrict which kinds of children are allowed:

```fr
// Any children allowed
column: {
    children: [
        text: { content: "Item 1" }
        button: { content: "Item 2" }
    ]
}

// Only 'tab' children allowed
tab_bar: {
    children: [
        tab: { content: "Home" }
        tab: { content: "Settings" }
    ]
}

// No children allowed
divider: {}
```

## Events

Events are callback props that fire in response to user interaction. They start with `on_` and take a function call as their value:

```fr
button: {
    content: "Save"
    on_click: saveData()
}

input: {
    value: email
    on_change: validateEmail()
    on_submit: submitForm()
    on_focus: logFocus()
    on_blur: logBlur()
}
```

Components also support lifecycle hooks: `on_mount`, `on_update`, `on_unmount`.

## Style Props

All components accept a `styles: { ... }` block. Most layout-oriented components accept the full set of layout styles; others accept a restricted subset. The style system includes:

| Category | Properties |
|----------|------------|
| **Layout** | `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height`, `flex`, `direction`, `align`, `justify`, `gap`, `aspect_ratio` |
| **Spacing** | `margin`, `margin_top`, `margin_bottom`, `margin_left`, `margin_right`, `padding`, `padding_top`, `padding_bottom`, `padding_left`, `padding_right` |
| **Appearance** | `background`, `color`, `font_size`, `font_weight`, `font_family`, `border`, `border_radius`, `opacity` |
| **Safe Area** | `safe_area: true/false` (defaults to `true`) |
| **Overflow** | `overflow: hidden/scroll/visible`, `overflow_x`, `overflow_y`, `clip_behavior` |
| **Text** | `text_overflow: ellipsis/clip/fade`, `max_lines`, `line_clamp` |
| **Image** | `fit: cover/contain/fill/fit_width/fit_height/none` |
| **Scroll** | `scroll_indicator`, `scroll_snap`, `scroll_enabled` |
| **Position** | `position`, `top`, `bottom`, `left`, `right`, `z_index` |

## The `plugin:` Component

The `plugin` component bridges to native plugin functionality. It requires `name` (plugin identifier) and `method` (the function to call) props, and has no children or style props:

```fr
plugin: { name: "frame_camera"  method: "captureImage" }
plugin: { name: "analytics"  method: "trackEvent" }
```

See the [Plugin Component](plugin.md) and [Plugin System](../plugin-system.md) docs for details.

## Component Categories

| Category | Count | Components |
|----------|-------|------------|
| Layout | 15 | row, column, container, stack, scaffold, card, divider, spacer, scroll_view, grid, list, form, item, accordion, timeline |
| Text/Content | 11 | text, button, icon, image, avatar, badge, chip, tag, banner, skeleton, table |
| Input | 14 | input, text_area, dropdown, switch, checkbox, radio, slider, stepper, search_bar, date_picker, time_picker, color_picker, rating, otp_input |
| Navigation | 8 | app_bar, bottom_navigation_bar, sidebar, floating_action_button, tab_bar, tab, bottom_sheet, modal |
| Feedback | 4 | toast, tooltip, progress_bar, progress_circle |
| Media | 7 | video_player, audio_player, lottie, web_view, map_view, camera_view, qr_scanner |
| Gesture | 4 | swipeable, draggable, refresh, long_press |
| Misc | 1 | plugin |
