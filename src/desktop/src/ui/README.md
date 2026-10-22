# Interface primitives

Button is adapted from Untitled UI React's MIT-licensed button at revision
`1d410cb8facde8d50869e7bbb5f3d048e2267466`, using React Aria for press, focus,
disabled behavior. Pending work uses a local text status and a disabled button,
so navigating away cannot leave an announcement referencing an unmounted control. Its API is narrowed to buttons; semantic CSS
replaces upstream Tailwind styles. Loading, decoration and default button type
retain the upstream pattern. The full upstream license is preserved in
`src/desktop/public/licenses/untitled-ui.txt`. No paid or proprietary components are used.

The remaining interface and chart components are original Apache-2.0 code.
Brand colors and focus treatments are shared through local CSS variables, with
system light/dark themes and reduced-motion support. No hosted font is required.
