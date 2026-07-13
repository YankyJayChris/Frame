# Media Components

Media components provide video/audio playback, animation rendering, embedded web browsing, maps, camera preview, and QR scanning. All accept the full set of layout styles.

---

## video_player

Video playback using native player (AVPlayer on iOS, ExoPlayer on Android).

```fr
video_player: { src: "https://example.com/video.mp4" }
video_player: {
    src: UserStore.video_url
    on_complete: handleVideoEnd()
    styles: { width: 100%  height: 300dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Video URL or asset path |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_complete` | No |

---

## audio_player

Audio playback using native player (AVAudioPlayer on iOS, MediaPlayer on Android).

```fr
audio_player: { src: "https://example.com/audio.mp3" }
audio_player: {
    src: UserStore.audio_url
    on_complete: trackFinished()
    styles: { width: 100% }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Audio URL or asset path |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_complete` | No |

---

## lottie

Lottie animation. Requires `lottie-ios` pod on iOS and Lottie Compose on Android.

```fr
lottie: { src: "https://example.com/animation.json" }
lottie: {
    src: "assets/animations/loading.json"
    on_complete: animationDone()
    styles: { width: 200dp  height: 200dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | String | **Yes** | — | Lottie JSON URL or asset path |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | `on_complete` | No |

---

## web_view

Embedded web browser component.

```fr
web_view: { url: "https://example.com" }
web_view: {
    url: UserStore.web_url
    styles: { width: 100%  height: 400dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `url` | String | **Yes** | — | URL to load |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | No |

---

## map_view

Embedded native map. Uses MapKit on iOS, Google Maps on Android.

```fr
map_view: { lat: 37.7749  lng: -122.4194 }
map_view: {
    lat: UserStore.latitude
    lng: UserStore.longitude
    styles: { width: 100%  height: 300dp  border_radius: 12dp }
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `lat` | Float | No | — | Latitude |
| `lng` | Float | No | — | Longitude |

| Styles | Events | Children |
|--------|--------|----------|
| All layout | — | No |

---

## camera_view

Live camera preview. Uses AVCaptureSession on iOS, CameraX on Android.

```fr
camera_view: { styles: { width: 100%  height: 300dp } }
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| All layout | — |

---

## qr_scanner

QR/barcode scanner with live preview.

```fr
qr_scanner: {
    styles: { width: 100%  height: 300dp }
    on_scan: handleQRCode()
}
```

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| — | — | — | — | No children |

| Styles | Events |
|--------|--------|
| All layout | `on_scan` |
