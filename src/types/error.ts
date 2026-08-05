/**
 * 镜像 Rust AppError 的序列化格式：
 * `{ "variant": "Db" | "NotFound" | "Validation" | "Io" | "Internal", "message": "..." }`
 */
export interface AppError {
  variant: "Db" | "NotFound" | "Validation" | "Io" | "Internal";
  message: string;
}

/** 从 invoke 抛出的未知错误中提取 AppError，或返回 fallback */
export function parseAppError(err: unknown): AppError {
  if (typeof err === "object" && err !== null && "variant" in err && "message" in err) {
    return err as unknown as AppError;
  }
  // Tauri 可能把错误包成 string
  if (typeof err === "string") {
    try {
      return JSON.parse(err) as AppError;
    } catch {
      // fall through
    }
  }
  return { variant: "Internal", message: String(err) };
}
