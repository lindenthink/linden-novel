import type { Editor, Range } from "@tiptap/core";

/**
 * 菜单项分类 —— 斜杠菜单与右键菜单共享
 */
export type MenuItemCategory =
  | "基础"
  | "标题"
  | "列表"
  | "样式"
  | "特殊"
  | "AI 助手";

/**
 * 菜单项定义
 * - range 存在时（斜杠菜单）：先删除触发文本再执行操作
 * - range 不存在时（右键菜单）：直接对当前选区/光标执行操作
 * - icon 使用 Carbon 图标类名（通过 UnoCSS presetIcons 渲染）
 */
export interface MenuItem {
  id: string;
  title: string;
  description: string;
  icon: string;
  category: MenuItemCategory;
  command: (props: { editor: Editor; range?: Range }) => void;
}

/**
 * 共享菜单项 —— 斜杠菜单和右键菜单共用同一份数据
 */
export const sharedMenuItems: MenuItem[] = [
  // ── 基础 ──
  {
    id: "paragraph",
    title: "正文",
    description: "普通段落文本",
    icon: "i-carbon-document",
    category: "基础",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.setParagraph().run();
    },
  },

  // ── 标题 ──
  {
    id: "heading-1",
    title: "一级标题",
    description: "大号分区标题",
    icon: "i-carbon-heading",
    category: "标题",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleHeading({ level: 1 }).run();
    },
  },
  {
    id: "heading-2",
    title: "二级标题",
    description: "中号章节标题",
    icon: "i-carbon-heading",
    category: "标题",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleHeading({ level: 2 }).run();
    },
  },
  {
    id: "heading-3",
    title: "三级标题",
    description: "小号子章节标题",
    icon: "i-carbon-heading",
    category: "标题",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleHeading({ level: 3 }).run();
    },
  },

  // ── 列表 ──
  {
    id: "bullet-list",
    title: "无序列表",
    description: "创建项目符号列表",
    icon: "i-carbon-list-bulleted",
    category: "列表",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleBulletList().run();
    },
  },
  {
    id: "ordered-list",
    title: "有序列表",
    description: "创建带编号的列表",
    icon: "i-carbon-list-numbered",
    category: "列表",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleOrderedList().run();
    },
  },
  {
    id: "task-list",
    title: "待办事项",
    description: "创建可勾选的任务列表",
    icon: "i-carbon-checkbox",
    category: "列表",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleTaskList().run();
    },
  },

  // ── 样式 ──
  {
    id: "blockquote",
    title: "引用",
    description: "引用其他内容",
    icon: "i-carbon-quotes",
    category: "样式",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleBlockquote().run();
    },
  },
  {
    id: "code-block",
    title: "代码块",
    description: "插入代码示例",
    icon: "i-carbon-code",
    category: "样式",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.toggleCodeBlock().run();
    },
  },

  // ── 特殊 ──
  {
    id: "scene-break",
    title: "分场线",
    description: "插入场景分割线",
    icon: "i-carbon-cut",
    category: "特殊",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.setSceneBreak().run();
    },
  },
  {
    id: "horizontal-rule",
    title: "水平线",
    description: "插入分隔线",
    icon: "i-carbon-horizontal-line-solid",
    category: "特殊",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.setHorizontalRule().run();
    },
  },

  // ── AI 助手 ──
  {
    id: "ai-continue",
    title: "AI 续写",
    description: "让 AI 继续当前内容",
    icon: "i-carbon-ai",
    category: "AI 助手",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.run();
      (editor as any).emit("aiAction", "continue");
    },
  },
  {
    id: "ai-expand",
    title: "AI 扩写",
    description: "让 AI 扩写当前内容，增加细节",
    icon: "i-carbon-ai",
    category: "AI 助手",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.run();
      (editor as any).emit("aiAction", "expand");
    },
  },
  {
    id: "ai-rewrite",
    title: "AI 改写",
    description: "让 AI 改写当前段落",
    icon: "i-carbon-ai",
    category: "AI 助手",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.run();
      (editor as any).emit("aiAction", "rewrite");
    },
  },
  {
    id: "ai-polish",
    title: "AI 润色",
    description: "让 AI 优化文字表达",
    icon: "i-carbon-ai",
    category: "AI 助手",
    command: ({ editor, range }) => {
      const chain = editor.chain().focus();
      if (range) chain.deleteRange(range);
      chain.run();
      (editor as any).emit("aiAction", "polish");
    },
  },
];

/** 斜杠菜单的分类顺序（块格式在前，AI 在后） */
export const slashCategoryOrder: MenuItemCategory[] = [
  "基础",
  "标题",
  "列表",
  "样式",
  "特殊",
  "AI 助手",
];

/** 右键菜单的分类顺序（AI 在前，块格式在后） */
export const contextCategoryOrder: MenuItemCategory[] = [
  "AI 助手",
  "基础",
  "标题",
  "列表",
  "样式",
  "特殊",
];

/** 按关键字过滤菜单项 */
export function filterItemsByQuery(items: MenuItem[], query: string): MenuItem[] {
  const lower = query.toLowerCase();
  return items.filter(
    (item) =>
      item.title.toLowerCase().includes(lower) ||
      item.description.toLowerCase().includes(lower)
  );
}

/** 按指定分类顺序分组 */
export function groupItemsByCategory(
  items: MenuItem[],
  order: MenuItemCategory[]
): { category: MenuItemCategory; items: MenuItem[] }[] {
  return order
    .map((category) => ({
      category,
      items: items.filter((item) => item.category === category),
    }))
    .filter((group) => group.items.length > 0);
}
