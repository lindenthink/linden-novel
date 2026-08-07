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
  safelist: [
    // DraggableHandle 动态注入
    "i-carbon-draggable",
    // BlockMenu
    "i-carbon-heading",
    "i-carbon-document",
    "i-carbon-list-bulleted",
    "i-carbon-list-numbered",
    "i-carbon-checkbox",
    "i-carbon-quotes",
    "i-carbon-code",
    "i-carbon-cut",
    "i-carbon-arrow-up",
    "i-carbon-arrow-down",
    "i-carbon-copy",
    "i-carbon-trash-can",
    // 共享菜单（斜杠 + 右键）
    "i-carbon-document",
    "i-carbon-horizontal-line-solid",
    "i-carbon-ai",
    // 右键菜单内联
    "i-carbon-text-bold",
    "i-carbon-text-italic",
    "i-carbon-text-underline",
    "i-carbon-text-color",
    "i-carbon-color-palette",
    "i-carbon-link",
    // SP4 长上下文
    "i-carbon-search-locate",
  ],
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
