import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, X } from "lucide-react";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { Dropdown } from "../../ui/Dropdown";
import { ShortcutInput } from "../ShortcutInput";
import { useSettings } from "../../../hooks/useSettings";
import { commands } from "@/bindings";
import type { ShortcutBinding } from "@/bindings";
import { toast } from "sonner";

const NONE_VALUE = "__none__";

/** Sort transcribe bindings: "transcribe" first, rest alphabetically */
function sortBindings(a: ShortcutBinding, b: ShortcutBinding): number {
  if (a.id === "transcribe") return -1;
  if (b.id === "transcribe") return 1;
  return a.id.localeCompare(b.id);
}

export const ShortcutBindingsCard: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [isAdding, setIsAdding] = useState(false);

  const bindings = getSetting("bindings") || {};
  const prompts = getSetting("post_process_prompts") || [];

  // Filter to transcribe-prefixed bindings only
  const transcribeBindings = Object.values(bindings)
    .filter(
      (b): b is ShortcutBinding =>
        !!b && (b.id === "transcribe" || b.id.startsWith("transcribe_")),
    )
    .sort(sortBindings);

  // Strategy dropdown options
  const strategyOptions = [
    { value: NONE_VALUE, label: t("settings.general.shortcuts.strategyNone") },
    ...prompts.map((p) => ({ value: p.id, label: p.name })),
  ];

  const handleStrategyChange = async (bindingId: string, value: string) => {
    const promptId = value === NONE_VALUE ? null : value;
    const result = await commands.updateBindingPrompt(bindingId, promptId);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  const handleAdd = async () => {
    setIsAdding(true);
    try {
      const result = await commands.addTranscribeBinding(
        "ctrl+alt+shift+f12",
        null,
      );
      if (result.status === "ok" && result.data.success) {
        await refreshSettings();
      } else if (result.status === "ok" && result.data.error) {
        toast.error(result.data.error);
      }
    } catch (error) {
      toast.error(String(error));
    } finally {
      setIsAdding(false);
    }
  };

  const handleRemove = async (bindingId: string) => {
    const result = await commands.removeTranscribeBinding(bindingId);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  const canDelete = (id: string) => id !== "transcribe" && id !== "cancel";

  return (
    <SettingsGroup title={t("settings.general.shortcuts.title")}>
      {transcribeBindings.map((binding) => (
        <div key={binding.id} className="flex items-center gap-2 px-3 py-1.5">
          <ShortcutInput shortcutId={binding.id} compact={true} />
          {prompts.length > 0 && (
            <Dropdown
              options={strategyOptions}
              selectedValue={binding.post_process_prompt_id || NONE_VALUE}
              onSelect={(value) => handleStrategyChange(binding.id, value)}
              className="flex-1 min-w-0"
            />
          )}
          {canDelete(binding.id) && (
            <button
              onClick={() => handleRemove(binding.id)}
              className="p-1 text-muted/40 hover:text-red-400 transition-colors rounded hover:bg-red-400/10"
              title={t("settings.general.shortcuts.remove")}
            >
              <X size={14} />
            </button>
          )}
        </div>
      ))}
      <button
        onClick={handleAdd}
        disabled={isAdding}
        className="flex items-center gap-1.5 w-full px-3 py-2 text-xs text-muted hover:text-text transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
      >
        <Plus size={13} />
        {t("settings.general.shortcuts.addNew")}
      </button>
    </SettingsGroup>
  );
};
