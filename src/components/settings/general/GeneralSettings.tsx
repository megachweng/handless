import React from "react";
import { useTranslation } from "react-i18next";
import { MicrophoneSelector } from "../MicrophoneSelector";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { VolumeSlider } from "../VolumeSlider";
import { OutputDeviceSelector } from "../OutputDeviceSelector";
import { ShowOverlay } from "../ShowOverlay";

export const GeneralSettings: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="max-w-3xl w-full space-y-8">
      <h1 className="sr-only">{t("sidebar.general")}</h1>

      <SettingsGroup title={t("settings.sound.title")}>
        <MicrophoneSelector descriptionMode="tooltip" grouped={true} />
        <VolumeSlider />
        <OutputDeviceSelector descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.advanced.groups.app")}>
        <ShowOverlay descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>
    </div>
  );
};
