# Animations

Frame provides an `animate:` block for defining property animations on components.

## Basic Animation

```fr
button: {
    content: "Animated"
    animate: {
        property: opacity
        from: 0
        to: 1
        duration: 300ms
        delay: 100ms
        easing: ease_in_out
        repeat: false
        auto_reverse: false
    }
}
```

## Fade In Animation

```fr
image: {
    src: "https://example.com/photo.jpg"
    styles: { width: 100%  height: 200dp }
    animate: {
        property: opacity
        from: 0.0
        to: 1.0
        duration: 500ms
        easing: ease_in
    }
}
```

## Scale Animation

```fr
button: {
    content: "Pulse"
    animate: {
        property: scale_x
        from: 1.0
        to: 1.1
        duration: 200ms
        easing: ease_in_out
        auto_reverse: true
        repeat: true
    }
}

button: {
    content: "Grow"
    animate: {
        property: scale_y
        from: 0.0
        to: 1.0
        duration: 300ms
        easing: spring
    }
}
```

## Rotation Animation

```fr
icon: {
    name: "arrow.clockwise"
    animate: {
        property: rotation
        from: 0
        to: 360
        duration: 1000ms
        easing: linear
        repeat: true
    }
}
```

## Translation Animation

```fr
card: {
    styles: { padding: 16dp }
    animate: {
        property: translation_x
        from: -100
        to: 0
        duration: 400ms
        easing: ease_out
    }
    children: [
        text: { content: "Slide in" }
    ]
}

container: {
    styles: { width: 100dp  height: 100dp  background: "#FF0000" }
    animate: {
        property: translation_y
        from: -200
        to: 0
        duration: 600ms
        easing: bounce
    }
}
```

## Multiple Animations

```fr
card: {
    styles: { padding: 16dp  background: "#FFFFFF" }
    animate: {
        property: opacity
        from: 0
        to: 1
        duration: 300ms
    }
    animate: {
        property: translation_y
        from: 20
        to: 0
        duration: 300ms
        delay: 100ms
    }
    children: [
        text: { content: "Fade and slide" }
    ]
}
```

## Animation Properties

| Property | Description | Example Values |
|----------|-------------|----------------|
| `opacity` | Fade in/out | `0` to `1` |
| `scale_x` | Horizontal scale | `0.0` to `1.0` |
| `scale_y` | Vertical scale | `0.0` to `1.0` |
| `rotation` | Rotation in degrees | `0` to `360` |
| `translation_x` | Horizontal position | `-100` to `0` |
| `translation_y` | Vertical position | `-200` to `0` |

## Easing Types

| Easing | Description |
|--------|-------------|
| `linear` | Constant speed |
| `ease_in` | Starts slow, accelerates |
| `ease_out` | Starts fast, decelerates |
| `ease_in_out` | Slow start and end |
| `bounce` | Bouncy overshoot effect |
| `spring` | Spring-like oscillation |

## Animation Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `property` | string | — | The property to animate |
| `from` | string/float | — | Starting value |
| `to` | string/float | — | Ending value |
| `duration` | time | `300ms` | Animation duration |
| `delay` | time | `0ms` | Delay before start |
| `easing` | string | `linear` | Easing curve |
| `repeat` | bool | `false` | Repeat indefinitely |
| `auto_reverse` | bool | `false` | Play forward then reverse |

Duration and delay values can be specified in `ms` (milliseconds) or `s` (seconds):

```fr
duration: 300ms
duration: 1s
duration: 500ms

delay: 100ms
delay: 0.5s
```
