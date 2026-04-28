import { create } from "zustand";
import { commands, type SttProviderInfo } from "@/bindings";

interface ModelsStore {
  providers: SttProviderInfo[];
  loading: boolean;
  error: string | null;
  initialized: boolean;
  initialize: () => Promise<void>;
  loadProviders: () => Promise<void>;
  setError: (error: string | null) => void;
  setLoading: (loading: boolean) => void;
}

export const useModelStore = create<ModelsStore>()((set, get) => ({
  providers: [],
  loading: true,
  error: null,
  initialized: false,

  setError: (error) => set({ error }),
  setLoading: (loading) => set({ loading }),

  loadProviders: async () => {
    try {
      const result = await commands.getAllSttProviders();
      if (result.status === "ok") {
        set({ providers: result.data, error: null });
      } else {
        set({ error: `Failed to load STT providers: ${result.error}` });
      }
    } catch (err) {
      set({ error: `Failed to load STT providers: ${err}` });
    } finally {
      set({ loading: false });
    }
  },

  initialize: async () => {
    if (get().initialized) return;
    await get().loadProviders();
    set({ initialized: true });
  },
}));
