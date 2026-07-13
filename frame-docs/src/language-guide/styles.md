# Styles

Every component supports a `styles:` block with over 60 style properties for layout, spacing, appearance, text, scroll behavior, and more.

## Basic Usage

```fr
container: {
    styles: {
        width: 100%
        height: 200dp
        background: "#F5F5F5"
        border_radius: 12dp
        padding: 16dp
        margin_top: 8dp
        overflow: hidden
        safe_area: true
        opacity: 0.9
    }
}
```

## Layout Properties

| Property       | Description                          | Example               |
|----------------|--------------------------------------|-----------------------|
| `width`        | Width dimension                      | `100%`, `200dp`, `50` |
| `height`       | Height dimension                     | `100%`, `300dp`       |
| `min_width`    | Minimum width                        | `100dp`               |
| `max_width`    | Maximum width                        | `500dp`               |
| `min_height`   | Minimum height                       | `100dp`               |
| `max_height`   | Maximum height                       | `500dp`               |
| `x`            | X position (absolute)                | `16dp`                |
| `y`            | Y position (absolute)                | `16dp`                |
| `flex`         | Flex grow factor                     | `1`                   |
| `flex_wrap`    | Flex wrap behavior                   | `wrap`, `nowrap`      |
| `direction`    | Flex direction                       | `row`, `column`       |
| `align`        | Cross-axis alignment                 | `center`, `stretch`   |
| `justify`      | Main-axis alignment                  | `space_between`       |
| `gap`          | Spacing between children             | `8dp`, `12dp`         |
| `aspect_ratio` | Width/height ratio                   | `16/9`, `1`           |

```fr
row: {
    styles: {
        width: 100%
        flex: 1
        direction: row
        align: center
        justify: space_between
        gap: 8dp
    }
}
```

## Spacing Properties

| Property         | Description        | Example  |
|------------------|--------------------|----------|
| `margin`         | All margins        | `16dp`   |
| `margin_top`     | Top margin         | `8dp`    |
| `margin_bottom`  | Bottom margin      | `8dp`    |
| `margin_left`    | Left margin        | `8dp`    |
| `margin_right`   | Right margin       | `8dp`    |
| `padding`        | All padding        | `16dp`   |
| `padding_top`    | Top padding        | `8dp`    |
| `padding_bottom` | Bottom padding     | `8dp`    |
| `padding_left`   | Left padding       | `8dp`    |
| `padding_right`  | Right padding      | `8dp`    |

## Appearance Properties

| Property        | Description           | Example                     |
|-----------------|-----------------------|-----------------------------|
| `background`    | Background color      | `"#FF0000"`, `$primary`     |
| `color`         | Text/foreground color | `"#333333"`, `$text`        |
| `font_size`     | Font size             | `16sp`, `18dp`              |
| `font_weight`   | Font weight           | `"bold"`, `"600"`           |
| `font_family`   | Font family name      | `"Inter"`, `"Helvetica"`    |
| `border`        | Border shorthand      | `"1px solid #CCC"`          |
| `border_radius` | Corner radius         | `8dp`, `12dp`               |
| `opacity`       | Opacity (0.0–1.0)     | `0.5`                       |
| `visible`       | Visibility toggle     | `true`, `false`             |
| `safe_area`     | Safe area insets      | `true` (default), `false`   |

```fr
text: {
    content: "Styled Text"
    styles: {
        font_size: 18sp
        font_weight: "bold"
        color: "#333333"
        font_family: "Inter"
        text_overflow: ellipsis
        max_lines: 2
    }
}
```

## Overflow Properties

| Property       | Values                              | Description                     |
|----------------|-------------------------------------|---------------------------------|
| `overflow`     | `hidden`, `scroll`, `visible`, `auto`, `scroll_x`, `scroll_y` | Content overflow behavior |
| `overflow_x`   | Same as above                       | Horizontal overflow only        |
| `overflow_y`   | Same as above                       | Vertical overflow only          |
| `clip_behavior`| `anti_aliased`, `hard`, `none`     | Clipping quality                |

```fr
container: {
    styles: {
        overflow: hidden
        clip_behavior: anti_aliased
        width: 200dp
        height: 200dp
    }
}
```

## Text Overflow Properties

| Property       | Values                    | Description              |
|----------------|---------------------------|--------------------------|
| `text_overflow`| `ellipsis`, `clip`, `fade`| Text truncation behavior |
| `max_lines`    | Integer                   | Maximum visible lines    |
| `line_clamp`   | Integer                   | Alias for `max_lines`    |

```fr
text: {
    content: "Very long text that should be truncated..."
    styles: {
        text_overflow: ellipsis
        max_lines: 1
    }
}
```

## Image Fit Properties

| Property | Values                                                        | Description       |
|----------|---------------------------------------------------------------|-------------------|
| `fit`    | `cover`, `contain`, `fill`, `fit_width`, `fit_height`, `none` | Image resize mode |

```fr
image: {
    src: "https://example.com/photo.jpg"
    styles: {
        width: 100%
        height: 200dp
        fit: cover
        border_radius: 12dp
    }
}
```

## Scroll Properties

| Property           | Values                         | Description              |
|--------------------|--------------------------------|--------------------------|
| `scroll_indicator` | `true`, `false`                | Show/hide scroll bar     |
| `scroll_snap`      | `start`, `center`, `end`, `none`| Scroll snap alignment   |
| `scroll_enabled`   | `true`, `false`                | Enable/disable scrolling |

```fr
scroll_view: {
    styles: {
        width: 100%
        height: 100%
        scroll_indicator: true
        scroll_snap: start
    }
    children: [ ... ]
}
```

## Responsive Breakpoints

Define breakpoints globally in `project.fr`:

```fr
:breakpoints { sm: 360dp  md: 600dp  lg: 900dp  xl: 1200dp }
```

Use breakpoint overrides directly in styles:

```fr
column: {
    styles: {
        width: 100%
        @md { width: 75% }
        @lg { width: 50% }
        padding: 8dp
        @md { padding: 16dp }
        @lg { padding: 24dp }
    }
}
```

Responsive arrays:

```fr
column: {
    styles: {
        width: [100%, @md: 75%, @lg: 50%]
        font_size: [14sp, @md: 16sp, @lg: 18sp]
    }
}
```

## Design Token References

Use `$variable` references to access design tokens defined in `:vars`:

```fr
column: {
    styles: {
        background: $surface
        padding: $spacing
        color: $primary
    }
}
```

## Platform Notes

| Platform | Layout Engine |
|----------|---------------|
| iOS      | Auto Layout + UIStackView |
| Android  | Jetpack Compose modifiers |
