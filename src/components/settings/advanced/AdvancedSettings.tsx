import React from "react";
import { useTranslation } from "react-i18next";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useAppVersion } from "@/hooks/useAppVersion";
import { AutostartToggle } from "../AutostartToggle";
import { StartHidden } from "../StartHidden";
import { ShowTrayIcon } from "../ShowTrayIcon";
import { ModelUnloadTimeoutSetting } from "../ModelUnloadTimeout";
import { ClipboardHandlingSetting } from "../ClipboardHandling";
import { AppendTrailingSpace } from "../AppendTrailingSpace";
import { TypingToolSetting } from "../TypingTool";
import { MuteWhileRecording } from "../MuteWhileRecording";
import { ThemeSelector } from "../ThemeSelector";
import { PasteMethodSetting } from "../PasteMethod";
import { AutoSubmit } from "../AutoSubmit";
import { AppLanguageSelector } from "../AppLanguageSelector";
import { KeyboardImplementationSelector } from "../debug/KeyboardImplementationSelector";
import { LogDirectory } from "../debug";
import { AppDataDirectory } from "../AppDataDirectory";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { Button } from "../../ui/Button";
import { ExportImportSettings } from "./ExportImportSettings";
import { ConfigFileSettings } from "./ConfigFileSettings";

export const AdvancedSettings: React.FC = () => {
  const { t } = useTranslation();
  const version = useAppVersion();

  return (
    <div className="max-w-3xl w-full space-y-8">
      <h1 className="sr-only">{t("sidebar.advanced")}</h1>

      <SettingsGroup title={t("settings.advanced.groups.app")}>
        <AppLanguageSelector descriptionMode="tooltip" grouped={true} />
        <ThemeSelector descriptionMode="tooltip" grouped={true} />
        <StartHidden descriptionMode="tooltip" grouped={true} />
        <AutostartToggle descriptionMode="tooltip" grouped={true} />
        <ShowTrayIcon descriptionMode="tooltip" grouped={true} />
        <ModelUnloadTimeoutSetting descriptionMode="tooltip" grouped={true} />
        <MuteWhileRecording descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.advanced.groups.output")}>
        <PasteMethodSetting descriptionMode="tooltip" grouped={true} />
        <AutoSubmit descriptionMode="tooltip" grouped={true} />
        <ClipboardHandlingSetting descriptionMode="tooltip" grouped={true} />
        <AppendTrailingSpace descriptionMode="tooltip" grouped={true} />
        <TypingToolSetting descriptionMode="tooltip" grouped={true} />
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

      <SettingsGroup title={t("settings.about.title")}>
        <SettingContainer
          title={t("settings.about.version.title")}
          description={t("settings.about.version.description")}
          grouped={true}
        >
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="text-sm font-mono">v{version}</span>
        </SettingContainer>
        <SettingContainer
          title={t("settings.about.sourceCode.title")}
          description={t("settings.about.sourceCode.description")}
          grouped={true}
        >
          <Button
            variant="secondary"
            size="default"
            onClick={() => openUrl("https://github.com/ElwinLiu/handless")}
          >
            {t("settings.about.sourceCode.button")}
          </Button>
        </SettingContainer>
        <SettingContainer
          title={t("settings.about.acknowledgments.handy.title")}
          description={t("settings.about.acknowledgments.handy.description")}
          grouped={true}
        >
          <span className="text-sm text-muted">
            {t("settings.about.acknowledgments.handy.details")}
          </span>
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
};
