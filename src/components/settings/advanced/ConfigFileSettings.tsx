import React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { SettingContainer } from "../../ui/SettingContainer";
import { commands } from "@/bindings";
import { useSettingsStore } from "@/stores/settingsStore";

export const ConfigFileSettings: React.FC = () => {
  const { t } = useTranslation();
  const refreshSettings = useSettingsStore((s) => s.refreshSettings);

  const openConfigFile = async () => {
    try {
      const result = await commands.openSettingsFile();
      if (result.status === "error") {
        toast.error(result.error);
      }
    } catch (error) {
      console.error("Failed to open settings file:", error);
    }
  };

  const reloadConfigFile = async () => {
    try {
      const result = await commands.reloadSettings();
      if (result.status === "error") {
        toast.error(result.error);
        return;
      }
      await refreshSettings();
      toast.success(t("settings.advanced.configFile.reloadSuccess"));
    } catch (error) {
      console.error("Failed to reload settings:", error);
    }
  };

  const buttonClass =
    "px-3 py-1 text-xs font-medium rounded-md bg-background-translucent border border-muted/20 text-muted hover:text-text hover:border-muted/40 transition-colors";

  return (
    <SettingContainer
      title={t("settings.advanced.configFile.title")}
      description={t("settings.advanced.configFile.description")}
      descriptionMode="tooltip"
      grouped={true}
    >
      <div className="flex gap-2">
        <button onClick={openConfigFile} className={buttonClass}>
          {t("settings.advanced.configFile.open")}
        </button>
        <button onClick={reloadConfigFile} className={buttonClass}>
          {t("settings.advanced.configFile.reload")}
        </button>
      </div>
    </SettingContainer>
  );
};
