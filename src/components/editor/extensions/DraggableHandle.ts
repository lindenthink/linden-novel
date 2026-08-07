import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { DecorationSet } from "@tiptap/pm/view";

export const DraggableHandle = Extension.create({
  name: "draggableHandle",

  addOptions() {
    return {
      handleWidth: 24,
    };
  },

  addProseMirrorPlugins() {
    // 在 addProseMirrorPlugins 中 this 是 Editor，但 Plugin.view() 中 this 是 Plugin 实例
    // 所以必须在此捕获 editor 引用，否则 view() 内部 this.editor 为 undefined
    const editor = this.editor;

    return [
      new Plugin({
        key: new PluginKey("draggableHandle"),
        state: {
          init: () => DecorationSet.empty,
          apply: (tr, old) => {
            if (tr.docChanged) {
              return DecorationSet.empty;
            }
            return old;
          },
        },
        view() {
          let hoveredNode: HTMLElement | null = null;
          let handleElement: HTMLElement | null = null;
          let hideTimer: ReturnType<typeof setTimeout> | null = null;
          let containerEl: HTMLElement | null = null;

          const createHandle = () => {
            const handle = document.createElement("div");
            handle.className = "block-handle-container";
            handle.innerHTML = `
              <div class="block-handle-drag" draggable="true" title="拖拽移动 / 点击打开菜单">
                <span class="i-carbon-draggable" style="font-size: 14px;"></span>
              </div>
            `;
            return handle;
          };

          // 获取定位容器（.editor-paper 或 .editor-scroll-area 或 editor 父元素）
          const getContainer = (): HTMLElement | null => {
            return (
              (editor.view.dom.closest(".editor-paper") as HTMLElement | null) ||
              (editor.view.dom.closest(".editor-scroll-area") as HTMLElement | null) ||
              editor.view.dom.parentElement
            );
          };

          // 用 getBoundingClientRect 计算位置 — handle 在 ProseMirror DOM 外部
          const updateHandlePosition = (node: HTMLElement) => {
            if (!handleElement || !node || !containerEl) return;

            const nodeRect = node.getBoundingClientRect();
            const containerRect = containerEl.getBoundingClientRect();

            handleElement.style.top = `${nodeRect.top - containerRect.top + containerEl.scrollTop}px`;
            handleElement.style.left = `${nodeRect.left - containerRect.left + containerEl.scrollLeft - 24}px`;
            handleElement.style.height = `${nodeRect.height}px`;
          };

          const hideHandle = () => {
            if (hideTimer) {
              clearTimeout(hideTimer);
              hideTimer = null;
            }
            if (handleElement) {
              handleElement.remove();
              handleElement = null;
            }
            hoveredNode = null;
          };

          // 延迟隐藏，给鼠标移动到 handle 上的时间
          const scheduleHide = (delay = 200) => {
            if (hideTimer) clearTimeout(hideTimer);
            hideTimer = setTimeout(() => {
              hideHandle();
            }, delay);
          };

          // 取消隐藏
          const cancelHide = () => {
            if (hideTimer) {
              clearTimeout(hideTimer);
              hideTimer = null;
            }
          };

          const showHandle = (node: HTMLElement) => {
            cancelHide();

            // 如果已经在当前节点，不需要重新创建
            if (handleElement && hoveredNode === node) {
              updateHandlePosition(node);
              return;
            }

            // 移除旧的 handle
            if (handleElement) {
              handleElement.remove();
              handleElement = null;
            }

            containerEl = getContainer();
            if (!containerEl) return;

            handleElement = createHandle();
            handleElement.addEventListener("click", handleClick);
            handleElement.addEventListener("dragstart", handleDragStart);
            // handle 上取消隐藏
            handleElement.addEventListener("mouseenter", cancelHide);
            handleElement.addEventListener("mouseleave", () => scheduleHide(100));

            // 追加到容器（ProseMirror DOM 外部）
            containerEl.appendChild(handleElement);

            hoveredNode = node;
            updateHandlePosition(node);
            handleElement.classList.add("visible");
          };

          const isBlockNode = (node: HTMLElement) => {
            return node.tagName === "P" ||
                   node.tagName === "H1" ||
                   node.tagName === "H2" ||
                   node.tagName === "H3" ||
                   node.tagName === "UL" ||
                   node.tagName === "OL" ||
                   node.tagName === "LI" ||
                   node.tagName === "BLOCKQUOTE" ||
                   node.tagName === "PRE" ||
                   node.tagName === "HR" ||
                   node.classList.contains("scene-break") ||
                   node.classList.contains("task-list") ||
                   node.classList.contains("task-item");
          };

          const getBlockElement = (node: HTMLElement) => {
            let current: HTMLElement | null = node;
            while (current && current !== editor.view.dom) {
              if (isBlockNode(current)) {
                return current;
              }
              current = current.parentElement;
            }
            return null;
          };

          const handleMouseMove = (event: MouseEvent) => {
            const target = event.target as HTMLElement;

            // 鼠标在 handle 上时不要隐藏
            if (handleElement && handleElement.contains(target)) {
              cancelHide();
              return;
            }

            // 取消任何挂起的隐藏
            cancelHide();

            const blockElement = getBlockElement(target);

            if (blockElement) {
              showHandle(blockElement);
            } else if (hoveredNode) {
              scheduleHide(100);
            }
          };

          const handleMouseLeave = (event: MouseEvent) => {
            // 鼠标移向 handle 时不要隐藏
            const relatedTarget = event.relatedTarget as HTMLElement;
            if (handleElement && relatedTarget && handleElement.contains(relatedTarget)) {
              return;
            }
            // 延迟隐藏，让鼠标有时间移动到 handle 上
            scheduleHide(200);
          };

          const handleClick = (event: MouseEvent) => {
            if (!handleElement || !hoveredNode) return;

            const target = event.target as HTMLElement;
            const dragBtn = handleElement.querySelector(".block-handle-drag");

            if (dragBtn?.contains(target)) {
              event.stopPropagation();
              event.preventDefault();
              editor.commands.focus();
              editor.emit("blockHandleClick" as any, {
                node: hoveredNode,
                pos: editor.view.posAtDOM(hoveredNode, 0),
              });
            }
          };

          const handleDragStart = (event: DragEvent) => {
            if (!handleElement || !hoveredNode) return;
            const target = event.target as HTMLElement;
            const dragBtn = handleElement.querySelector(".block-handle-drag");
            if (!dragBtn?.contains(target)) return;

            event.dataTransfer!.effectAllowed = "move";
            event.dataTransfer!.setData("text/plain", "block-drag");

            editor.emit("blockDragStart" as any, {
              node: hoveredNode,
              pos: editor.view.posAtDOM(hoveredNode, 0),
            });
          };

          const handleDrop = (event: DragEvent) => {
            event.preventDefault();
            const target = event.target as HTMLElement;
            const blockElement = getBlockElement(target);
            if (blockElement && hoveredNode && hoveredNode !== blockElement) {
              const from = editor.view.posAtDOM(hoveredNode, 0);
              const to = editor.view.posAtDOM(blockElement, 0);
              const node = editor.state.doc.nodeAt(from);
              if (node) {
                editor.chain()
                  .focus()
                  .deleteRange({ from, to: from + node.nodeSize })
                  .insertContentAt(to, [node.toJSON()])
                  .run();
              }
            }
          };

          const handleDragOver = (event: DragEvent) => {
            event.preventDefault();
            if (hoveredNode) {
              hoveredNode.classList.add("drag-over");
              setTimeout(() => {
                hoveredNode?.classList.remove("drag-over");
              }, 100);
            }
          };

          let editorDom: HTMLElement | null = null;
          let scrollContainer: HTMLElement | null = null;
          const handleScroll = () => {
            if (hoveredNode && handleElement) {
              updateHandlePosition(hoveredNode);
            }
          };

          // 在 view() 中直接初始化 — Tiptap v3 不会自动调用返回对象的 init()
          const dom = editor.view.dom;
          editorDom = dom;
          containerEl = getContainer();
          dom.addEventListener("mousemove", handleMouseMove);
          dom.addEventListener("mouseleave", handleMouseLeave);
          dom.addEventListener("drop", handleDrop);
          dom.addEventListener("dragover", handleDragOver);

          // 监听滚动事件，更新 handle 位置
          scrollContainer = dom.closest(".editor-scroll-area");
          if (scrollContainer) {
            scrollContainer.addEventListener("scroll", handleScroll);
          }

          // 注入拖拽样式
          const styleEl = document.createElement("style");
          styleEl.textContent = `
            .block-handle-drag { cursor: grab; }
            .block-handle-drag:active { cursor: grabbing; }
            .drag-over { outline: 2px dashed #3b82f6; outline-offset: 2px; }
          `;
          document.head.appendChild(styleEl);

          return {
            update: () => {
              // ProseMirror 更新后，hoveredNode 可能已不存在
              // 更新 handle 位置（如果还在显示）
              if (hoveredNode && handleElement && containerEl) {
                // 检查 hoveredNode 是否还在 DOM 中
                if (!document.body.contains(hoveredNode)) {
                  hideHandle();
                } else {
                  updateHandlePosition(hoveredNode);
                }
              }
            },
            destroy: () => {
              if (hideTimer) {
                clearTimeout(hideTimer);
                hideTimer = null;
              }
              if (editorDom) {
                editorDom.removeEventListener("mousemove", handleMouseMove);
                editorDom.removeEventListener("mouseleave", handleMouseLeave);
                editorDom.removeEventListener("drop", handleDrop);
                editorDom.removeEventListener("dragover", handleDragOver);
                editorDom = null;
              }
              if (scrollContainer) {
                scrollContainer.removeEventListener("scroll", handleScroll);
                scrollContainer = null;
              }
              if (handleElement) {
                handleElement.remove();
                handleElement = null;
              }
            },
          };
        },
      }),
    ];
  },
});

export default DraggableHandle;
