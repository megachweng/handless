import { type } from "@tauri-apps/plugin-os";
import { type OSType } from "../lib/utils/keyboard";

/**
 * Get the current OS type for keyboard handling.
 * This is a simple wrapper - type() is synchronous.
 */
export function useOsType(): OSType {
  const osType = type();
  return osType === "windows" || osType === "linux" ? osType : "unknown";
}
