import { useMessage } from "naive-ui";

/**
 * 统一错误处理：捕获 API 调用错误并显示友好提示
 */
export function useApiError() {
  const message = useMessage();

  function handleError(error: unknown, fallback: string = "操作失败") {
    let msg = fallback;

    if (error instanceof Error) {
      // Tauri invoke 错误通常包含 Rust 端的 AppError 信息
      const errStr = error.message;

      // 尝试解析 JSON 格式的错误（来自 Rust 的 AppError）
      try {
        const parsed = JSON.parse(errStr);
        if (parsed.variant && parsed.message) {
          msg = parsed.message;
        } else {
          msg = errStr;
        }
      } catch {
        // 不是 JSON，直接使用原始消息
        msg = errStr;
      }
    }

    message.error(msg);
    console.error("[API Error]", error);
  }

  return { handleError };
}
