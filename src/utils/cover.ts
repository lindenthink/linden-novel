import { convertFileSrc } from "@tauri-apps/api/core";
import { appDataDir, join } from "@tauri-apps/api/path";

// 模块级缓存：首次调用发起一次 IPC，后续复用，避免每张卡片重复查询 appDataDir
const appDataDirPromise = appDataDir();

/**
 * 将 DB 中存储的相对封面路径（如 `covers/xxx.png`）解析为可在 <img src> 中使用的 URL。
 *
 * 后端把图片复制到 `appDataDir/covers/{uuid}.{ext}`，DB 仅存相对路径；
 * 前端拼接绝对路径后通过 `convertFileSrc` 转成 asset 协议 URL 渲染。
 */
export async function resolveCoverUrl(
  coverPath: string | null | undefined,
): Promise<string | null> {
  if (!coverPath) return null;
  try {
    const base = await appDataDirPromise;
    const abs = await join(base, coverPath);
    return convertFileSrc(abs);
  } catch {
    return null;
  }
}
