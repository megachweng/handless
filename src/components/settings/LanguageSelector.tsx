import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { ResetButton } from "../ui/ResetButton";
import { Dropdown } from "../ui";
import { useSettings } from "../../hooks/useSettings";
import { LANGUAGES } from "../../lib/constants/languages";

interface LanguageSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  supportedLanguages?: string[];
}

export const LanguageSelector: React.FC<LanguageSelectorProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
  supportedLanguages,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, resetSetting, isUpdating } = useSettings();

  const selectedLanguage = getSetting("selected_language") || "auto";

  const options = useMemo(() => {
    const available =
      !supportedLanguages || supportedLanguages.length === 0
        ? LANGUAGES
        : LANGUAGES.filter(
            (lang) =>
              lang.value === "auto" || supportedLanguages.includes(lang.value),
          );
    return available.map((lang) => ({ value: lang.value, label: lang.label }));
  }, [supportedLanguages]);

  const handleLanguageSelect = async (languageCode: string) => {
    await updateSetting("selected_language", languageCode);
  };

  const handleReset = async () => {
    await resetSetting("selected_language");
  };

  return (
    <SettingContainer
      title={t("settings.general.language.title")}
      description={t("settings.general.language.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="flex items-center space-x-1">
        <Dropdown
          selectedValue={selectedLanguage}
          options={options}
          onSelect={handleLanguageSelect}
          placeholder={t("settings.general.language.auto")}
          disabled={isUpdating("selected_language")}
          className="min-w-[200px]"
          searchable
          searchPlaceholder={t("settings.general.language.searchPlaceholder")}
        />
        <ResetButton
          onClick={handleReset}
          disabled={isUpdating("selected_language")}
        />
      </div>
    </SettingContainer>
  );
};
