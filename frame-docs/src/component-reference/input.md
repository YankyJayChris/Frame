# Input Components

Input components let users enter, select, and manipulate data. All input components accept the full set of layout styles.

---

## input

Single-line text input field.

```fr
input: { value: searchQuery  placeholder: "Search..." }
input: {
    value: email
    placeholder: "Enter your email"
    validate: required | email
    on_error: showError()
    on_change: validateEmail()
    on_submit: submitForm()
    on_focus: logFocus()
    on_blur: logBlur()
    styles: { padding: 12dp  border_radius: 8dp  border: "1px solid #CCC"  width: 100% }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Current input value |
| `placeholder` | String | No | — | Placeholder text |
| `validate` | Expression | No | — | Validation rules |
| `on_error` | String | No | — | Error handler |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change`, `on_submit`, `on_focus`, `on_blur` | No |

---

## text_area

Multi-line text input.

```fr
text_area: { value: bio  placeholder: "Tell us about yourself..." }
text_area: {
    value: description
    placeholder: "Enter description"
    lines: 5
    validate: required | min_length(10)
    on_change: validateDescription()
    styles: { width: 100%  height: 120dp  padding: 12dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Current text |
| `placeholder` | String | No | — | Placeholder |
| `lines` | Int | No | — | Number of visible lines |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change`, `on_submit` | No |

---

## dropdown

Dropdown / select menu. Children are rendered as the option list.

```fr
dropdown: {
    value: selectedOption
    validate: required
    on_change: handleSelect()
    children: [
        text: { content: "Option A" }
        text: { content: "Option B" }
        text: { content: "Option C" }
    ]
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Currently selected value |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change`, `on_select` | Yes (any — rendered as options) |

---

## switch

Toggle switch. Uses `value` for checked state (`checked` is a synonym).

```fr
switch: { value: notificationsEnabled }
switch: { value: UserStore.notifications_enabled  on_change: wait:UserStore.toggleNotifications() }
switch: { checked: darkMode  on_change: toggleDarkMode() }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Bool | No | — | Checked state |
| `checked` | Bool | No | — | Synonym for `value` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## checkbox

Checkbox input.

```fr
checkbox: { value: agreeToTerms }
checkbox: { value: UserStore.opted_in  on_change: wait:UserStore.setOptIn() }
checkbox: { checked: isSelected  on_change: toggleSelect() }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Bool | No | — | Checked state |
| `checked` | Bool | No | — | Synonym for `value` |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## radio

Radio button for single-selection groups.

```fr
radio: { selected: isChosen }
radio: { selected: UserStore.selected == "option_a"  on_click: wait:UserStore.selectOption("option_a") }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `selected` | Bool | No | — | Whether this radio is selected |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_click` | No |

---

## slider

Range slider for numeric values.

```fr
slider: { value: volume }
slider: {
    value: UserStore.volume
    min: 0
    max: 100
    on_change: wait:UserStore.setVolume()
    styles: { width: 80%  padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Float | No | — | Current value |
| `min` | Float | No | — | Minimum value |
| `max` | Float | No | — | Maximum value |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## stepper

Increment/decrement stepper with + and - buttons.

```fr
stepper: { value: quantity }
stepper: {
    value: UserStore.item_count
    on_increment: wait:UserStore.increment()
    on_decrement: wait:UserStore.decrement()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Int | No | — | Current value |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_increment`, `on_decrement` | No |

---

## search_bar

Search input with magnifying glass icon and clear button.

```fr
search_bar: { value: query  placeholder: "Search..." }
search_bar: {
    value: UserStore.search_query
    placeholder: "Search users..."
    on_change: wait:UserStore.search()
    on_submit: executeSearch()
    styles: { width: 100%  padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Search query |
| `placeholder` | String | No | — | Placeholder text |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change`, `on_submit` | No |

---

## date_picker

Date picker. On iOS uses inline date picker; on Android uses Material DatePicker.

```fr
date_picker: { value: selectedDate }
date_picker: {
    value: UserStore.birth_date
    validate: required
    on_change: wait:UserStore.setDate()
    styles: { padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Date string |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## time_picker

Time picker. On iOS uses wheels style; on Android uses Material TimeInput.

```fr
time_picker: { value: selectedTime }
time_picker: {
    value: UserStore.reminder_time
    validate: required
    on_change: wait:UserStore.setTime()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Time string |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## color_picker

Color picker. On iOS uses UIColorWell; on Android renders a button showing selected color.

```fr
color_picker: { value: selectedColor }
color_picker: {
    value: UserStore.theme_color
    on_change: wait:UserStore.setColor()
    styles: { padding: 8dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | String | No | — | Hex color string |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## rating

Star rating display/interaction.

```fr
rating: { value: 3 }
rating: {
    value: UserStore.rating
    max: 5
    on_change: wait:UserStore.setRating()
}
rating: { value: 4  max: 10  styles: { padding: 8dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | Int | No | — | Current rating |
| `max` | Int | No | `5` | Maximum stars |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_change` | No |

---

## otp_input

One-time password input. Shows individual digit fields.

```fr
otp_input: { length: 6 }
otp_input: {
    length: 4
    validate: required | length(4)
    on_complete: verifyOTP()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `length` | Int | No | `6` | Number of digits |
| `validate` | Expression | No | — | Validation rules |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_complete` | No |
