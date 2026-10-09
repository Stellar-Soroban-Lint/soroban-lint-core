import DefaultTheme from "vitepress/theme";

/**
 * Theme entry point.
 *
 * Everything visual is the default VitePress theme plus `custom.css`. No
 * component overrides, no `enhanceApp` registration: the Telegram icon in the
 * header and footer is a plain data object in `config.mts`, which VitePress
 * renders through its own `VPIcon` without needing a Vue component here.
 */

import "./custom.css";

export default {
  extends: DefaultTheme,
};