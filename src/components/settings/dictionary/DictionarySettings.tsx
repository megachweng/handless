import React, { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Info } from "@phosphor-icons/react";
import { useSettings } from "@/hooks/useSettings";
import { useModelStore } from "@/stores/modelStore";
import { Textarea } from "@/components/ui/Textarea";
import { SettingsGroup } from "@/components/ui/SettingsGroup";
import { SettingContainer } from "@/components/ui/SettingContainer";
import { TagListInput } from "./TagListInput";

const ProviderSupportNotice: React.FC = () => {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const { providers } = useModelStore();

  const providerId = settings?.stt_provider_id ?? "local";

  const message = useMemo(() => {
    if (providerId === "local") {
      return t("dictionary.providerSupport.localProvider");
    }

    const provider = providers.find((p) => p.id === providerId);
    if (!provider) return null;

    const supportsTerms = provider.supports_dictionary_terms ?? false;
    const supportsContext = provider.supports_dictionary_context ?? false;

    if (supportsTerms && supportsContext) {
      return t("dictionary.providerSupport.supportsBoth", {
        provider: provider.name,
      });
    }
    if (supportsTerms) {
      return t("dictionary.providerSupport.supportsTermsOnly", {
        provider: provider.name,
      });
    }
    if (supportsContext) {
      return t("dictionary.providerSupport.supportsContextOnly", {
        provider: provider.name,
      });
    }
    return t("dictionary.providerSupport.supportsNeither", {
      provider: provider.name,
    });
  }, [providerId, providers, t]);

  if (!message) return null;

  return (
    <div className="flex items-start gap-2 px-3 py-2 rounded-lg bg-accent/5 border border-accent/10 text-xs text-text/60">
      <Info className="w-3.5 h-3.5 mt-0.5 shrink-0 text-accent" />
      <span>{message}</span>
    </div>
  );
};

const DictionaryContext: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const context = getSetting("dictionary_context") || "";
  const [localContext, setLocalContext] = useState(context);

  useEffect(() => {
    setLocalContext(context);
  }, [context]);

  return (
    <SettingContainer
      title={t("dictionary.context.title")}
      description={t("dictionary.context.description")}
      descriptionMode="tooltip"
      grouped
      layout="stacked"
    >
      <Textarea
        variant="compact"
        className="resize-none"
        rows={3}
        value={localContext}
        onChange={(e) => setLocalContext(e.target.value)}
        onBlur={() => {
          if (localContext !== context) {
            updateSetting("dictionary_context", localContext);
          }
        }}
        placeholder={t("dictionary.context.placeholder")}
        disabled={isUpdating("dictionary_context")}
      />
    </SettingContainer>
  );
};

export const DictionarySettings: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="max-w-3xl w-full mx-auto space-y-8">
      <h1 className="sr-only">{t("dictionary.title")}</h1>

      <ProviderSupportNotice />

      <SettingsGroup title={t("dictionary.terms.title")}>
        <TagListInput
          settingKey="dictionary_terms"
          title={t("dictionary.terms.title")}
          description={t("dictionary.terms.description")}
          placeholder={t("dictionary.terms.placeholder")}
          addLabel={t("dictionary.terms.add")}
          removeAriaLabel={(term) => t("dictionary.terms.remove", { term })}
          duplicateMessage={(term) =>
            t("dictionary.terms.duplicate", { term })
          }
          maxLength={100}
        />
      </SettingsGroup>

      <SettingsGroup title={t("dictionary.context.title")}>
        <DictionaryContext />
      </SettingsGroup>

      <SettingsGroup title={t("dictionary.customWords.title")}>
        <TagListInput
          settingKey="custom_words"
          title={t("dictionary.customWords.title")}
          description={t("dictionary.customWords.description")}
          placeholder={t("settings.advanced.customWords.placeholder")}
          addLabel={t("settings.advanced.customWords.add")}
          removeAriaLabel={(word) =>
            t("settings.advanced.customWords.remove", { word })
          }
          duplicateMessage={(word) =>
            t("settings.advanced.customWords.duplicate", { word })
          }
          maxLength={50}
          allowSpaces={false}
          sanitize={(v) => v.replace(/[<>"'&]/g, "")}
          inputClassName="max-w-40"
        />
      </SettingsGroup>
    </div>
  );
};
