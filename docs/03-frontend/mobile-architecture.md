# Mobile Architecture — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-FE-002`                               |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-FE-001, DOC-ARCH-001                  |

---

## 1. Technology Stack

| Library | Version | Purpose |
|---------|---------|---------|
| **React Native** | 0.73+ | Cross-platform mobile framework |
| **Expo** | SDK 51+ | Build tooling, native module management, OTA updates |
| **TypeScript** | 5+ | Type safety |
| **Expo Router** | 3+ | File-based routing |
| **Zustand** | 4+ | State management (shared patterns with web) |
| **TanStack Query** | 5+ | Server state management |
| **expo-av** | Latest | Audio/video playback |
| **expo-camera** | Latest | Camera access for video calls |
| **expo-file-system** | Latest | File operations for media |
| **expo-notifications** | Latest | Push notification handling |
| **expo-secure-store** | Latest | Secure token storage |
| **react-native-webrtc** | Latest | WebRTC for voice/video calls |

## 2. Project Structure

```
frontend/mobile/
├── app.json                        # Expo configuration
├── eas.json                        # EAS Build configuration
├── package.json
├── tsconfig.json
├── babel.config.js
├── metro.config.js
├── assets/
│   ├── icon.png
│   ├── splash.png
│   └── adaptive-icon.png
├── src/
│   ├── app/                        # Expo Router (file-based routing)
│   │   ├── _layout.tsx             # Root layout (providers, auth guard)
│   │   ├── index.tsx               # Entry redirect
│   │   ├── (auth)/                 # Auth screens (no bottom tab)
│   │   │   ├── _layout.tsx
│   │   │   ├── login.tsx
│   │   │   ├── register.tsx
│   │   │   └── verify.tsx
│   │   ├── (main)/                 # Authenticated screens (bottom tab)
│   │   │   ├── _layout.tsx         # Bottom tab navigator
│   │   │   ├── conversations/
│   │   │   │   ├── index.tsx       # Conversation list
│   │   │   │   └── [id].tsx        # Chat screen
│   │   │   ├── calls/
│   │   │   │   └── index.tsx       # Call history
│   │   │   ├── contacts/
│   │   │   │   └── index.tsx       # Contact list
│   │   │   └── settings/
│   │   │       └── index.tsx       # Settings screen
│   │   └── call/
│   │       └── [id].tsx            # Active call screen (full screen)
│   │
│   ├── components/                 # Reusable components
│   │   ├── ui/                     # Primitives (Button, Input, Avatar)
│   │   ├── chat/                   # Chat components (MessageBubble, InputBar)
│   │   ├── call/                   # Call UI (IncomingCallModal, CallControls)
│   │   └── layout/                 # Screen wrappers, headers
│   │
│   ├── hooks/                      # Custom hooks (shared patterns with web)
│   │   ├── useAuth.ts
│   │   ├── useWebSocket.ts
│   │   ├── useConversation.ts
│   │   ├── usePresence.ts
│   │   ├── useTyping.ts
│   │   ├── useCall.ts
│   │   └── useMediaUpload.ts
│   │
│   ├── stores/                     # Zustand stores (same API as web)
│   │   ├── authStore.ts
│   │   ├── conversationStore.ts
│   │   ├── presenceStore.ts
│   │   └── callStore.ts
│   │
│   ├── lib/                        # Non-React utilities
│   │   ├── api.ts                  # REST API client
│   │   ├── websocket.ts            # WebSocket client
│   │   ├── webrtc.ts               # WebRTC peer connection
│   │   ├── notifications.ts        # FCM token management
│   │   ├── storage.ts              # expo-secure-store wrappers
│   │   └── permissions.ts          # Permission request helpers
│   │
│   └── types/                      # TypeScript types (shared with web)
│       ├── api.ts
│       ├── websocket.ts
│       ├── models.ts
│       └── errors.ts
```

## 3. Code Sharing with Web

### Shared (Same Code)

| Module | Why Shareable |
|--------|--------------|
| `types/` | TypeScript interfaces are platform-agnostic |
| `stores/` (Zustand) | State logic is platform-agnostic |
| WebSocket protocol handling | JSON parsing, event routing |
| API client (fetch-based) | React Native supports `fetch` |
| Business logic in hooks | Auth flows, message handling logic |

### Platform-Specific

| Concern | Web | Mobile |
|---------|-----|--------|
| Token storage | Memory + HttpOnly cookie | `expo-secure-store` |
| Push notifications | Web Push API | `expo-notifications` (FCM) |
| File picker | `<input type="file">` | `expo-image-picker`, `expo-document-picker` |
| Camera/microphone | `navigator.mediaDevices` | `expo-camera`, `expo-av` |
| Background behavior | Service Worker (limited) | Native background services |
| Navigation | Next.js router | Expo Router |
| UI components | HTML/CSS | React Native Views |

### Sharing Strategy

Use a **shared types package** or a `shared/` directory:
```
frontend/
├── shared/                    # Platform-agnostic code
│   ├── types/                 # Shared TypeScript types
│   ├── constants/             # Shared constants (API routes, limits)
│   └── utils/                 # Shared utility functions
├── web/                       # Web-specific
└── mobile/                    # Mobile-specific
```

## 4. Mobile-Specific Concerns

### Background WebSocket

On Android, WebSocket connections are killed when the app is backgrounded. Solution:
1. When app goes to background → close WebSocket gracefully
2. Rely on FCM push notifications for new message alerts
3. When app returns to foreground → reconnect WebSocket + sync

### Push Notifications (FCM)

```typescript
// Register for push notifications
import * as Notifications from 'expo-notifications';

async function registerForPushNotifications() {
  const { status } = await Notifications.requestPermissionsAsync();
  if (status !== 'granted') return null;
  
  const token = await Notifications.getExpoPushTokenAsync({
    projectId: Constants.expoConfig.extra.eas.projectId,
  });
  
  // Send token to backend
  await api.patch(`/devices/${deviceId}`, {
    push_token: token.data,
    push_provider: 'FCM',
  });
  
  return token;
}
```

### Offline Message Queue

Same pattern as web — messages sent offline are queued in AsyncStorage and sent on reconnect with original `client_message_id` for idempotency.

### Permissions

| Permission | When Requested | Fallback if Denied |
|-----------|---------------|-------------------|
| Notifications | First launch or first message received | No push notifications; messages available on app open |
| Camera | First video call or first photo capture | Cannot make video calls; cannot take photos |
| Microphone | First audio/video call | Cannot make calls |
| Storage | First media download | Cannot save media to device |

**Rule:** Request permissions at the moment of use, not on first launch. Explain why the permission is needed before the system dialog appears.

## 5. Build & Distribution

### Development
```bash
# Start Expo dev server
cd frontend/mobile
npx expo start

# Run on Android emulator
npx expo run:android

# Run on physical device (Expo Go or dev build)
npx expo start --dev-client
```

### Production Build (EAS Build)
```bash
# Configure EAS
npx eas-cli build:configure

# Build Android APK/AAB
npx eas-cli build --platform android --profile production

# Build preview (internal testing)
npx eas-cli build --platform android --profile preview
```

### Distribution

| Channel | Method |
|---------|--------|
| Internal testing | EAS Build → download APK directly |
| Beta testing | Google Play Console (internal/closed testing track) |
| Production | Google Play Console (production track) |
| OTA updates | EAS Update (JS-only updates without store review) |

## 6. Mobile Security

| Concern | Mitigation |
|---------|-----------|
| Token storage | `expo-secure-store` (Android Keystore backed) |
| Certificate pinning | Verify server certificate in production builds |
| Root/jailbreak detection | Warn user but don't block (portfolio project) |
| Screen capture | Allow (not a banking app) |
| Clipboard security | Clear sensitive data from clipboard after paste |
| Deep link validation | Validate all deep link parameters before navigation |

---

*Next: [shared-code.md](shared-code.md) · [push-notifications.md](push-notifications.md)*
