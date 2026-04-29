import React from "react";
import { useTranslation } from "react-i18next";
import { AutostartToggle } from "../AutostartToggle";
import { StartHidden } from "../StartHidden";
import { ShowTrayIcon } from "../ShowTrayIcon";
import { ClipboardHandlingSetting } from "../ClipboardHandling";
import { AppendTrailingSpace } from "../AppendTrailingSpace";
import { MuteWhileRecording } from "../MuteWhileRecording";
import { ThemeSelector } from "../ThemeSelector";
import { PasteMethodSetting } from "../PasteMethod";
import { AutoSubmit } from "../AutoSubmit";
import { AppLanguageSelector } from "../AppLanguageSelector";
import { KeyboardImplementationSelector } from "../debug/KeyboardImplementationSelector";
import { LogDirectory } from "../debug";
import { AppDataDirectory } from "../AppDataDirectory";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { ExportImportSettings } from "./ExportImportSettings";
import { ConfigFileSettings } from "./ConfigFileSettings";

export const AdvancedSettings: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="max-w-3xl w-full space-y-8">
      <h1 className="sr-only">{t("sidebar.advanced")}</h1>

      <SettingsGroup title={t("settings.advanced.groups.app")}>
        <AppLanguageSelector descriptionMode="tooltip" grouped={true} />
        <ThemeSelector descriptionMode="tooltip" grouped={true} />
        <StartHidden descriptionMode="tooltip" grouped={true} />
        <AutostartToggle descriptionMode="tooltip" grouped={true} />
        <ShowTrayIcon descriptionMode="tooltip" grouped={true} />
        <MuteWhileRecording descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.advanced.groups.output")}>
        <PasteMethodSetting descriptionMode="tooltip" grouped={true} />
        <AutoSubmit descriptionMode="tooltip" grouped={true} />
        <ClipboardHandlingSetting descriptionMode="tooltip" grouped={true} />
        <AppendTrailingSpace descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.advanced.groups.configuration")}>
        <KeyboardImplementationSelector
          descriptionMode="tooltip"
          grouped={true}
        />
        <ConfigFileSettings />
      </SettingsGroup>

      <SettingsGroup title={t("settings.advanced.groups.data")}>
        <ExportImportSettings />
        <AppDataDirectory descriptionMode="tooltip" grouped={true} />
        <LogDirectory grouped={true} />
      </SettingsGroup>
    </div>
  );
};
