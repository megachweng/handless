import type { SttProviderInfo } from "@/bindings";

/**
 * Filter providers to "My Models": downloaded/custom local models + cloud providers with an API key.
 */
export function filterMyProviders(
  providers: SttProviderInfo[],
  sttApiKeys: Partial<Record<string, string>> | undefined,
): SttProviderInfo[] {
  return providers.filter((p) => {
    if (p.backend.type === "Cloud") {
      const apiKey = sttApiKeys?.[p.id];
      return !!(apiKey && apiKey.trim());
    }
    return p.backend.is_downloaded || p.backend.is_custom;
  });
}
