import {
  defineConfig,
  presetUno,
  presetAttributify,
  presetTypography,
} from "unocss";

// UnoCSS 配置：原子化排版 + 属性化模式 + 排版（编辑器 prose）+ 暗色 class 策略
export default defineConfig({
  presets: [presetUno(), presetAttributify(), presetTypography()],
  dark: "class",
  theme: {
    colors: {
      linden: {
        primary: "#7c5cff",
      },
    },
  },
});
