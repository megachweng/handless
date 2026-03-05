import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Dropdown, type DropdownOption } from "@/components/ui";
import { LANGUAGES } from "@/lib/constants/languages";

interface LanguageFilterProps {
  value: string;
  onChange: (value: string) => void;
}

export const LanguageFilter: React.FC<LanguageFilterProps> = ({
  value,
  onChange,
}) => {
  const { t } = useTranslation();

  const options = useMemo<DropdownOption[]>(() => {
    const allOption: DropdownOption = {
      value: "all",
      label: t("settings.models.filters.allLanguages"),
    };
    const langOptions = LANGUAGES.filter((lang) => lang.value !== "auto").map(
      (lang) => ({
        value: lang.value,
        label: lang.label,
      }),
    );
    return [allOption, ...langOptions];
  }, [t]);

  return (
    <Dropdown
      selectedValue={value}
      options={options}
      onSelect={onChange}
      placeholder={t("settings.models.filters.allLanguages")}
      searchable
      searchPlaceholder={t("settings.general.language.searchPlaceholder")}
    />
  );
};
