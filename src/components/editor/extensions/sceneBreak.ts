import { Node, mergeAttributes, nodeInputRule } from "@tiptap/core";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import SceneBreakNodeView from "./SceneBreakNodeView.vue";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    sceneBreak: {
      setSceneBreak: () => ReturnType;
    };
  }
}

export const SceneBreak = Node.create({
  name: "sceneBreak",

  group: "block",

  atom: true,

  addCommands() {
    return {
      setSceneBreak:
        () =>
        ({ commands }) => {
          return commands.insertContent({ type: this.name });
        },
    };
  },

  addInputRules() {
    return [
      nodeInputRule({
        find: /^\* \* \*$/,
        type: this.type,
      }),
    ];
  },

  addKeyboardShortcuts() {
    return {
      "Mod-Shift--": () => this.editor.commands.setSceneBreak(),
    };
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="scene-break"]',
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "div",
      mergeAttributes(HTMLAttributes, { "data-type": "scene-break" }),
      "* * *",
    ];
  },

  addNodeView() {
    return VueNodeViewRenderer(SceneBreakNodeView);
  },
});

export default SceneBreak;
