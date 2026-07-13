# Validation

Frame provides a validation system for form inputs using `:validation` schemas and inline validation rules.

## Validation Schemas

Define reusable validation schemas with the `:validation` keyword:

```fr
:validation LoginForm {
    email:    required | email
    password: required | min_length(8)
}

:validation RegistrationForm {
    name:       required | min_length(2) | max_length(100)
    email:      required | email | max_length(255)
    password:   required | min_length(8)
    age:        optional | min(0) | max(150)
    website:    optional | url
    bio:        optional | max_length(500)
}
```

## Using a Schema on a Form

```fr
form: {
    schema: LoginForm
    children: [
        input: { value: email  placeholder: "Email" }
        input: { value: password  placeholder: "Password"  secure: true }
        button: { content: "Login"  on_click: submitLogin() }
    ]
}
```

## Inline Validation

Apply validation rules directly on input components:

```fr
input: {
    value: email
    placeholder: "Email"
    validate: required | email
    on_error: showEmailError()
}

input: {
    value: username
    placeholder: "Username"
    validate: required | min_length(3) | max_length(20)
}
```

## Validation Rules Reference

| Rule            | Description                        | Example               |
|-----------------|------------------------------------|-----------------------|
| `required`      | Field must not be empty            | `required`            |
| `optional`      | Field is optional                  | `optional`            |
| `email`         | Valid email format                 | `email`               |
| `min(n)`        | Minimum numeric value              | `min(0)`              |
| `max(n)`        | Maximum numeric value              | `max(150)`            |
| `min_length(n)` | Minimum string length              | `min_length(8)`       |
| `max_length(n)` | Maximum string length              | `max_length(255)`     |
| `pattern(regex)`| Custom regex pattern               | `pattern("^[A-Z].*")` |
| `url`           | Valid URL format                   | `url`                 |
| `match(field)`  | Must match another field's value   | `match(password)`     |

## Full Form with Validation

```fr
:validation UserSchema {
    name:       required | min_length(2) | max_length(100)
    email:      required | email
    age:        optional | min(0) | max(150)
    agree:      required
}

page: {
    name: "Register"
    route: "/register"
    state: {
        formName: string = ""
        formEmail: string = ""
        formAge: string = ""
        formAgree: bool = false
        errors: object = {}
    }
    children: [
        form: {
            schema: UserSchema
            children: [
                column: {
                    styles: { padding: 16dp  gap: 12dp }
                    children: [
                        text: { content: "Register"  styles: { font_size: 24sp  font_weight: "bold" } }
                        input: {
                            value: state.formName
                            placeholder: "Name"
                            validate: required | min_length(2)
                        }
                        input: {
                            value: state.formEmail
                            placeholder: "Email"
                            validate: required | email
                        }
                        input: {
                            value: state.formAge
                            placeholder: "Age"
                            validate: optional | min(0) | max(150)
                        }
                        checkbox: {
                            value: state.formAgree
                            label: "I agree to the terms"
                            validate: required
                        }
                        button: {
                            content: "Submit"
                            on_click: submitForm()
                        }
                    ]
                }
            ]
        }
    ]

    fn submitForm: () => {
        log.info("Form submitted")
    }
}
```

## Error Handling

```fr
input: {
    value: email
    placeholder: "Email"
    validate: required | email
    on_error: showValidationError()
}

fn showValidationError: (message: string) => {
    UserStore.validation_error = message
}
```

## Platform Behavior

Validation rules are compiled to platform-native validation:

| Platform | Implementation |
|----------|----------------|
| Android  | TextInputLayout with built-in validators |
| iOS      | UITextField delegate validation in view controller |
