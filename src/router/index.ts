import { createRouter, createWebHashHistory } from "vue-router";

// Tauri 桌面端用 hash history，避免 file/tauri 协议下 history 模式空白页
const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("../views/ProjectListView.vue"),
    },
    {
      path: "/project/:id",
      name: "editor",
      component: () => import("../views/EditorView.vue"),
    },
  ],
});

export default router;
