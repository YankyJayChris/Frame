# Events and Lifecycle

Frame provides event handlers for user interactions and lifecycle hooks for components and pages.

## Component Events

### Click Events

```fr
button: {
    content: "Click me"
    on_click: handleClick()
}

icon: {
    name: "gearshape"
    on_click: openSettings()
}

card: {
    styles: { padding: 16dp }
    children: [ text: { content: "Tap me" } ]
    on_click: handleCardTap()
}
```

### Change Events

```fr
input: {
    value: searchQuery
    on_change: handleSearch()
}

switch: {
    value: notificationsEnabled
    on_change: wait:UserStore.toggleNotifications()
}

slider: {
    value: volume
    on_change: adjustVolume()
}

dropdown: {
    value: selectedOption
    on_change: handleSelect()
    on_select: handleSelect()
}

checkbox: {
    value: agreeToTerms
    on_change: toggleAgreement()
}

rating: {
    value: score
    on_change: wait:Store.setRating()
}
```

### Focus and Blur Events

```fr
input: {
    value: email
    placeholder: "Email"
    on_focus: logFocus()
    on_blur: logBlur()
}

input: {
    value: password
    placeholder: "Password"
    on_focus: highlightField()
    on_blur: validateField()
}
```

### Submit Events

```fr
input: {
    value: query
    on_submit: executeSearch()
}

form: {
    schema: LoginForm
    on_submit: handleLogin()
    children: [ ... ]
}
```

### Gesture Events

```fr
swipeable: {
    on_swipe: handleSwipe()
    children: [ card: { children: [ text: { content: "Swipe me" } ] } ]
}

draggable: {
    on_drag: handleDrag()
    children: [ text: { content: "Drag me" } ]
}

long_press: {
    on_long_press: showContextMenu()
    children: [ text: { content: "Press and hold" } ]
}
```

### Scroll Events

```fr
scroll_view: {
    styles: { width: 100%  height: 100% }
    children: [ ... ]
}

column: {
    styles: { overflow: scroll  height: 100% }
    on_scroll: handleScroll()
    on_scroll_end: handleScrollEnd()
    children: [ ... ]
}
```

### Media Events

```fr
video_player: {
    src: "https://example.com/video.mp4"
    on_complete: handleVideoEnd()
}

audio_player: {
    src: "https://example.com/audio.mp3"
    on_complete: trackFinished()
}

lottie: {
    src: "assets/animations/loading.json"
    on_complete: animationDone()
}
```

### Input Component Events

```fr
search_bar: {
    value: query
    placeholder: "Search..."
    on_change: wait:Store.search()
    on_submit: executeSearch()
}

otp_input: {
    length: 6
    on_complete: verifyOTP()
}

stepper: {
    value: quantity
    on_increment: increment()
    on_decrement: decrement()
}

refresh: {
    refreshing: UserStore.is_refreshing
    on_refresh: wait:UserStore.refresh()
    children: [ ... ]
}

qr_scanner: {
    styles: { width: 100%  height: 300dp }
    on_scan: handleQRCode()
}
```

## Component Lifecycle Hooks

```fr
column: {
    on_mount: loadInitialData()     // fires once after first render
    on_update: refreshList()        // fires when watched dependency changes
    watch: UserStore.items          // dependency for on_update
    on_unmount: cancelRequests()    // fires when node is removed
    children: [ ... ]
}
```

| Event       | Timing                              |
|-------------|--------------------------------------|
| `on_mount`  | After first render                   |
| `on_update` | When `watch` dependency changes      |
| `watch`     | Expression to watch for changes      |
| `on_unmount`| When node is removed from tree       |

## Page Lifecycle Hooks

```fr
page: {
    name: "Dashboard"
    route: "/dashboard"
    before_enter: checkAuth()          // called before transition
    before_leave: saveState()          // called when navigating away
    on_mount: loadData()               // called when fully visible
    on_background: pauseUpdates()      // app goes to background
    on_foreground: resumeUpdates()     // app returns to foreground
    on_unmount: cleanup()              // called on page dispose
    children: [ ... ]
}
```

| Event            | Timing                               |
|------------------|--------------------------------------|
| `before_enter`   | Before page transition completes     |
| `before_leave`   | When navigating away from page       |
| `on_mount`       | After page is fully visible          |
| `on_background`  | App moves to background              |
| `on_foreground`  | App returns to foreground            |
| `on_unmount`     | Page is removed from navigation stack|

## Platform Mapping

| Event       | Android                            | iOS                            |
|-------------|------------------------------------|--------------------------------|
| `on_click`  | `Modifier.clickable`               | `UITapGestureRecognizer`       |
| `on_change` | `onValueChange` callback           | `.valueChanged` event          |
| `on_focus`  | `Modifier.onFocusChanged`          | `textFieldDidBeginEditing`     |
| `on_blur`   | `Modifier.onFocusChanged`          | `textFieldDidEndEditing`       |
| `on_mount`  | `LaunchedEffect(Unit)`             | `viewDidAppear`                |
| `on_update` | `LaunchedEffect(key)`              | `viewDidLayoutSubviews`        |
| `on_unmount`| `DisposableEffect.onDispose`       | `viewDidDisappear`             |
