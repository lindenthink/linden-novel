import { Extension } from "@tiptap/core";
import {
  Suggestion,
  exitSuggestion,
  type SuggestionProps,
  type SuggestionKeyDownProps,
} from "@tiptap/suggestion";
import {
  sharedMenuItems,
  filterItemsByQuery,
  type MenuItem,
} from "../menuItems";

export const SlashCommand = Extension.create({
  name: "slashCommand",

  addOptions() {
    return {
      suggestion: {
        char: "/",
        // v3: command 接收 { editor, range, props }，props 是选中的 item
        command: ({ editor, range, props }: any) => {
          props.command({ editor, range });
        },
        items: ({ query }: { query: string }) => {
          return filterItemsByQuery(sharedMenuItems, query);
        },
        render: () => {
          let component: {
            props: SuggestionProps<MenuItem>;
            selectedIndex: number;
            items: MenuItem[];
          } | null = null;
          let popup: HTMLElement | null = null;
          let unmount: (() => void) | null = null;

          const renderMenu = () => {
            if (!component || !popup) return;

            const { items, selectedIndex } = component;
            if (!items.length) {
              popup.innerHTML = '<div class="slash-menu-empty">没有匹配的命令</div>';
              return;
            }

            // 按分类分组（sharedMenuItems 已按分类排序，插入顺序即分类顺序）
            const groups: Record<string, MenuItem[]> = {};
            const categoryOrder: string[] = [];
            items.forEach((item) => {
              if (!groups[item.category]) {
                groups[item.category] = [];
                categoryOrder.push(item.category);
              }
              groups[item.category].push(item);
            });

            let html = "";
            let globalIndex = -1;

            categoryOrder.forEach((category) => {
              html += `<div class="slash-menu-group">${category}</div>`;
              groups[category].forEach((item) => {
                globalIndex++;
                const isSelected = globalIndex === selectedIndex;
                html += `
                  <div class="slash-menu-item ${isSelected ? "selected" : ""}" data-index="${globalIndex}">
                    <span class="slash-icon"><span class="${item.icon} text-base"></span></span>
                    <div class="slash-content">
                      <div class="slash-title">${item.title}</div>
                      <div class="slash-desc">${item.description}</div>
                    </div>
                  </div>
                `;
              });
            });

            popup.innerHTML = html;
            popup.className =
              "slash-menu-container bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 py-2 min-w-[220px] max-w-[280px]";

            // 绑定点击：用 mousedown + preventDefault 防止编辑器失焦触发 onExit
            popup.querySelectorAll(".slash-menu-item").forEach((el: Element) => {
              (el as HTMLElement).addEventListener("mousedown", (e: MouseEvent) => {
                e.preventDefault();
                e.stopPropagation();
                if (!component) return;
                const index = parseInt((el as HTMLElement).dataset.index || "0");
                const item = component.items[index];
                if (item) {
                  // v3: command 只接收 item，内部会调用 options.command({ editor, range, props: item })
                  component.props.command(item);
                }
              });
            });

            // 滚动到选中项，保持可见
            const selectedEl = popup.querySelector(".slash-menu-item.selected");
            if (selectedEl) {
              (selectedEl as HTMLElement).scrollIntoView({ block: "nearest" });
            }
          };

          return {
            onStart: (props: SuggestionProps<MenuItem>) => {
              popup = document.createElement("div");
              popup.className = "slash-menu-container";

              component = {
                props,
                selectedIndex: 0,
                items: props.items,
              };

              // v3: 使用 mount 托管定位 + 外部点击关闭
              unmount = props.mount(popup);

              renderMenu();
            },
            onUpdate: (props: SuggestionProps<MenuItem>) => {
              if (!popup || !component) return;
              component.props = props;
              component.items = props.items;
              component.selectedIndex = 0;
              renderMenu();
            },
            onKeyDown: (props: SuggestionKeyDownProps) => {
              if (!component) return false;

              if (props.event.key === "Escape") {
                exitSuggestion(props.view);
                return true;
              }

              if (props.event.key === "ArrowDown") {
                props.event.preventDefault();
                component.selectedIndex =
                  (component.selectedIndex + 1) % component.items.length;
                renderMenu();
                return true;
              }

              if (props.event.key === "ArrowUp") {
                props.event.preventDefault();
                component.selectedIndex =
                  (component.selectedIndex - 1 + component.items.length) %
                  component.items.length;
                renderMenu();
                return true;
              }

              if (props.event.key === "Enter") {
                props.event.preventDefault();
                const item = component.items[component.selectedIndex];
                if (item) {
                  component.props.command(item);
                }
                return true;
              }

              return false;
            },
            onExit: () => {
              unmount?.();
              unmount = null;
              popup?.remove();
              popup = null;
              component = null;
            },
          };
        },
      },
    };
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        ...this.options.suggestion,
      }),
    ];
  },
});

export default SlashCommand;
