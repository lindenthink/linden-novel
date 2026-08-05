import {
  defineConfig,
  presetUno,
  presetAttributify,
  presetTypography,
  presetIcons,
} from "unocss";

// UnoCSS 配置：原子化排版 + 属性化模式 + 排版（编辑器 prose）+ 图标 + 暗色 class 策略
export default defineConfig({
  presets: [
    presetUno(),
    presetAttributify(),
    presetTypography(),
    presetIcons({
      scale: 1.2,
      warn: true,
      collections: {
        carbon: () => import("@iconify-json/carbon/icons.json").then((i) => i.default),
      },
    }),
  ],
  dark: "class",
  theme: {
    colors: {
      linden: {
        primary: "#7c5cff",
      },
      gray: {
        850: "#1a1a2e",
      },
    },
  },

});
