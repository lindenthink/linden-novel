import { ref } from "vue";
import { createDiscreteApi } from "naive-ui";

// 独立 message 实例（脱离 NMessageProvider 上下文，避免在非 setup 上下文调用报错）
const { message } = createDiscreteApi(["message"]);

export type UpdaterState =
  | "idle"
  | "checking"
  | "available"
  | "not-available"
  | "downloading"
  | "installed"
  | "error";

export interface UpdateInfo {
  version: string;
  notes?: string;
  pubDate?: string;
}

const LAST_CHECK_KEY = "linden_last_update_check";
const THROTTLE_HOURS = 24;

// 模块级单例（跨组件共享）
const state = ref<UpdaterState>("idle");
const updateInfo = ref<UpdateInfo | null>(null);
const progress = ref(0);
const errorMsg = ref("");
// 是否显示 UpdateDialog（由 App.vue 持有 dialog 实例，启动检查与手动检查都共用此 ref）
const showDialog = ref(false);

// 持有 Update 实例用于 downloadAndInstall（不响应式化，避免代理破坏其内部状态）
let pendingUpdate: Awaited<
  ReturnType<typeof import("@tauri-apps/plugin-updater")["check"]>
> | null = null;

function isThrottled(): boolean {
  const last = localStorage.getItem(LAST_CHECK_KEY);
  if (!last) return false;
  const lastTime = parseInt(last, 10);
  if (Number.isNaN(lastTime)) return false;
  const elapsedHours = (Date.now() - lastTime) / (1000 * 60 * 60);
  return elapsedHours < THROTTLE_HOURS;
}

export function useUpdater() {
  async function checkForUpdates(silent = false) {
    // 静默模式下节流，避免重复请求 GitHub
    if (silent && isThrottled()) {
      return;
    }

    state.value = "checking";
    errorMsg.value = "";
    updateInfo.value = null;
    pendingUpdate = null;

    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();

      // 无论结果如何都记录检查时间，便于节流
      localStorage.setItem(LAST_CHECK_KEY, Date.now().toString());

      if (update) {
        state.value = "available";
        pendingUpdate = update;
        updateInfo.value = {
          version: update.version,
          notes: update.body,
          pubDate: update.date,
        };
        // 检查到新版自动弹 dialog（silent 启动检查也弹，符合"有更新即提示"语义）
        showDialog.value = true;
      } else {
        state.value = "not-available";
        if (!silent) {
          message.success("当前已是最新版本");
        }
      }
    } catch (e) {
      state.value = "error";
      errorMsg.value = e?.toString() ?? "未知错误";
      if (silent) {
        console.warn("[updater] silent check failed:", e);
      } else {
        message.error(`检查更新失败：${errorMsg.value}`);
      }
    }
  }

  async function downloadAndInstall() {
    if (!pendingUpdate) {
      state.value = "error";
      errorMsg.value = "无可下载的更新";
      return;
    }

    state.value = "downloading";
    progress.value = 0;
    errorMsg.value = "";

    try {
      let contentLength = 0;
      let downloaded = 0;
      await pendingUpdate.downloadAndInstall((event: any) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data?.contentLength ?? 0;
            downloaded = 0;
            break;
          case "Progress":
            downloaded += event.data?.chunkLength ?? 0;
            if (contentLength > 0) {
              progress.value = Math.min(
                99,
                Math.floor((downloaded / contentLength) * 100),
              );
            }
            break;
          case "Finished":
            progress.value = 100;
            break;
        }
      });
      state.value = "installed";
    } catch (e) {
      state.value = "error";
      errorMsg.value = e?.toString() ?? "下载安装失败";
      message.error(`下载安装失败：${errorMsg.value}`);
    }
  }

  async function relaunch() {
    try {
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      console.error("[updater] relaunch failed:", e);
      message.error("重启失败，请手动重启应用");
    }
  }

  function reset() {
    state.value = "idle";
    errorMsg.value = "";
    updateInfo.value = null;
    progress.value = 0;
    pendingUpdate = null;
    showDialog.value = false;
  }

  return {
    state,
    updateInfo,
    progress,
    errorMsg,
    showDialog,
    checkForUpdates,
    downloadAndInstall,
    relaunch,
    reset,
  };
}
