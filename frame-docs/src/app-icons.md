# App Icons

App icons are the visual representation of your Frame application on the device home screen. Frame automatically generates platform-specific icon assets for iOS and Android during the build process.

## Overview

Every Frame app needs an app icon. You can:
- Use the **default Frame icon** (lime green with black Frame logo)
- Provide your **own custom icon** (SVG, PNG, or JPEG)

Icons are automatically generated at all required sizes for each platform when you run `frame build`.

## Default Icon

Frame includes a beautiful default app icon that works across all platforms:

```json
{
  "name": "my-app",
  "bundle_id": "com.example.myapp",
  "version": "1.0.0"
}
```

The default icon:
- **Color**: Lime green (#BCFB70) with black icon
- **Format**: Scalable vector (SVG)
- **Size**: 1080×1080 px
- **Location**: `assets/icons/frame-default.svg`

## Custom App Icon

### Configuration

Add the `icon` field to your `frame.config.json`:

```json
{
  "name": "my-app",
  "bundle_id": "com.example.myapp",
  "version": "1.0.0",
  "icon": "assets/my-app-icon.svg"
}
```

The `icon` path is **relative to your project root**.

### Supported Formats

| Format | Best For | Recommended Size | Notes |
|--------|----------|------------------|-------|
| **SVG** | Vector designs | 1024×1024 px | Scales perfectly to all sizes |
| **PNG** | Raster images | 1024×1024 px or larger | Lossless compression |
| **JPEG** | Photographs | 1024×1024 px or larger | Lossy (converts to PNG) |

### Recommended Specifications

#### Dimensions
- **Square**: Must be square (width = height)
- **Minimum**: 1024×1024 px (for raster formats)
- **Recommended**: 1024×1024 px or larger

#### Design Guidelines

**Safe Area**
- Keep important details in the center 75% of the icon
- Avoid relying on transparency for critical elements
- Allow for rounding on iOS and adaptive shapes on Android

**Colors & Contrast**
- Ensure good contrast against the home screen
- Test on both light and dark backgrounds
- Avoid pure white or pure black backgrounds (use neutral tones)

**Transparency**
- SVG and PNG support transparency
- JPEG does not support transparency (opaque white background added)
- For best results with transparency, use **SVG or PNG**

#### Example SVG Icon

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
  <!-- Background -->
  <rect width="1024" height="1024" fill="#2563EB" rx="256"/>
  
  <!-- Icon design -->
  <circle cx="512" cy="512" r="350" fill="#93C5FD"/>
  
  <!-- Optional: Text or additional elements -->
  <text x="512" y="550" font-size="200" font-weight="bold" 
        fill="white" text-anchor="middle">MY</text>
</svg>
```

## Generated Icon Files

When you run `frame build`, icons are automatically generated and placed in the build output directories.

### Android Icons

```
build/android/app/src/main/res/
├── mipmap-ldpi/
│   ├── ic_launcher.png          (144×144 px, 0.75x density)
│   └── ic_launcher_round.png    (rounded variant)
├── mipmap-mdpi/
│   ├── ic_launcher.png          (192×192 px, 1x density)
│   └── ic_launcher_round.png
├── mipmap-hdpi/
│   ├── ic_launcher.png          (288×288 px, 1.5x density)
│   └── ic_launcher_round.png
├── mipmap-xhdpi/
│   ├── ic_launcher.png          (384×384 px, 2x density)
│   └── ic_launcher_round.png
├── mipmap-xxhdpi/
│   ├── ic_launcher.png          (576×576 px, 3x density)
│   └── ic_launcher_round.png
├── mipmap-xxxhdpi/
│   ├── ic_launcher.png          (768×768 px, 4x density)
│   └── ic_launcher_round.png
└── values/
    └── colors.xml               (icon background colors)
```

**Densities Supported**
- `ldpi` (Low-density): 0.75× baseline
- `mdpi` (Medium-density): 1× baseline (192×192 px)
- `hdpi` (High-density): 1.5× baseline
- `xhdpi` (Extra-high-density): 2× baseline
- `xxhdpi` (Extra-extra-high-density): 3× baseline
- `xxxhdpi` (Extra-extra-extra-high-density): 4× baseline

### iOS Icons

```
build/ios/Runner/Assets.xcassets/AppIcon.appiconset/
├── Contents.json                (metadata for Xcode)
├── app_icon_20_at_2x.png        (40×40 px, Notification iPhone)
├── app_icon_20_at_3x.png        (60×60 px, Notification iPhone 6+)
├── app_icon_29_at_2x.png        (58×58 px, Settings iPhone)
├── app_icon_29_at_3x.png        (87×87 px, Settings iPhone 6+)
├── app_icon_40_at_2x.png        (80×80 px, Spotlight iPhone)
├── app_icon_40_at_3x.png        (120×120 px, Spotlight iPhone 6+)
├── app_icon_60_at_2x.png        (120×120 px, App iPhone)
├── app_icon_60_at_3x.png        (180×180 px, App iPhone 6+)
├── app_icon_76_at_2x.png        (152×152 px, iPad)
├── app_icon_76_at_3x.png        (228×228 px, iPad Pro)
├── app_icon_83_5_at_2x.png      (167×167 px, iPad Pro)
└── app_icon_1024_at_1x.png      (1024×1024 px, App Store)
```

**Sizes Supported**
- iPhone Notifications: 20pt, 40pt (scales to 40×40, 80×80 px)
- iPhone Settings: 29pt (scales to 58×58, 87×87 px)
- iPhone Spotlight: 40pt (scales to 80×80, 120×120 px)
- iPhone App Icon: 60pt (scales to 120×120, 180×180 px)
- iPad: 76pt (scales to 152×152, 228×228 px)
- iPad Pro: 83.5pt (scales to 167×167 px)
- App Store: 1024pt (1024×1024 px)

## Build Process

Icons are automatically generated during the build process:

```bash
frame build
```

This command:
1. Reads your icon source (default or custom from `frame.config.json`)
2. Validates the icon format and dimensions
3. Generates iOS icon assets in `build/ios/`
4. Generates Android icon assets in `build/android/`
5. Creates platform-specific metadata (Contents.json, colors.xml)

### Build Output

```
✓ Build complete in 2.34s — 287 file(s) generated
  Android: build/android/  iOS: build/ios/
  Run: frame deploy android  OR  frame deploy ios
```

## Icon Workflows

### Use Default Icon

1. Create a new project:
   ```bash
   frame start my-app
   ```

2. Build without specifying an icon:
   ```bash
   frame build
   ```

The default Frame icon is automatically used.

### Add Custom Icon

1. Place your icon file in your project:
   ```bash
   cp my-logo.svg assets/app-icon.svg
   ```

2. Update `frame.config.json`:
   ```json
   {
     "name": "my-app",
     "bundle_id": "com.example.myapp",
     "icon": "assets/app-icon.svg"
   }
   ```

3. Build:
   ```bash
   frame build
   ```

### Change Icon Later

1. Replace the icon file:
   ```bash
   cp new-icon.png assets/app-icon.png
   ```

2. Update path in `frame.config.json` if filename changed:
   ```json
   {
     "icon": "assets/app-icon.png"
   }
   ```

3. Rebuild:
   ```bash
   frame build
   ```

### Test Generated Icons

1. After build, icons are in:
   - Android: `build/android/app/src/main/res/mipmap-*/`
   - iOS: `build/ios/Runner/Assets.xcassets/AppIcon.appiconset/`

2. Deploy to simulator:
   ```bash
   frame deploy android
   frame deploy ios
   ```

3. Check home screen to verify icon appearance

## Troubleshooting

### Icon Not Appearing in App

**Check icon file exists**
```bash
ls -la assets/app-icon.svg
```

**Verify frame.config.json path**
```json
{
  "icon": "assets/app-icon.svg"
}
```

**Rebuild project**
```bash
rm -rf build/
frame build
```

### Icon Looks Distorted

**For SVG:**
- Ensure viewBox attribute is correct: `viewBox="0 0 1024 1024"`
- Keep design centered in the canvas

**For PNG/JPEG:**
- Must be square (width = height)
- Use at least 1024×1024 px
- Avoid compression artifacts

### Icon Generation Warnings

Check build output for messages like:
```
warning: could not render icon at size 192x192: File not found
```

**Common causes:**
- Icon file path is wrong (relative to project root)
- File permissions issue
- Unsupported format

**Solution:**
1. Verify path: `ls assets/your-icon.svg`
2. Check permissions: `chmod 644 assets/your-icon.svg`
3. Try another format (SVG > PNG > JPEG)

## Advanced: Manual Icon Editing

If you need fine-grained control:

1. Generate the default build:
   ```bash
   frame build
   ```

2. Manually edit icons in:
   - `build/android/app/src/main/res/mipmap-*/`
   - `build/ios/Runner/Assets.xcassets/AppIcon.appiconset/`

3. Your changes persist in the build/ directory

4. To regenerate from source:
   ```bash
   rm -rf build/
   frame build
   ```

## Best Practices

✅ **Do**
- Start with a 1024×1024 px square design
- Use SVG for vector designs (scales perfectly)
- Test on real devices or simulators
- Keep important content in the center 75%
- Use clear, recognizable shapes
- Maintain good contrast with backgrounds

❌ **Don't**
- Use non-square images (they'll be stretched)
- Rely solely on transparency for key elements
- Use text that's too small (won't be readable)
- Deploy without testing on actual devices
- Use pure white or pure black backgrounds
- Assume the same icon works equally well on all densities

## Example Projects

Check out the built-in examples:

- **blog-app**: Uses default Frame icon
- **profile**: Uses default Frame icon

Both are in the `examples/` directory and demonstrate icon generation during build.

## See Also

- [Icon System](icon-system.md) — Component icons used in UI
- [CLI Reference](cli-reference.md) — `frame build` command options
- [Getting Started](getting-started.md) — Project setup
