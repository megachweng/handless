import React, { useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, Cloud } from "@phosphor-icons/react";
import { getTranslatedModelName } from "../../lib/utils/modelTranslation";
import { useModelStore } from "../../stores/modelStore";
import { useSettings } from "../../hooks/useSettings";
import ModelDropdown from "./ModelDropdown";

interface ModelSelectorProps {
  onError?: (error: string) => void;
}

const ModelSelector: React.FC<ModelSelectorProps> = ({ onError }) => {
  const { t } = useTranslation();
  const { providers } = useModelStore();
  const { settings, setSttProvider } = useSettings();
  const [showDropdown, setShowDropdown] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const sttProviderId = settings?.stt_provider_id ?? "soniox";
  const selectedProvider = useMemo(
    () => providers.find((provider) => provider.id === sttProviderId),
    [providers, sttProviderId],
  );

  const handleProviderSelect = async (providerId: string) => {
    setShowDropdown(false);
    try {
      await setSttProvider(providerId);
    } catch (err) {
      console.error(`Failed to set STT provider to ${providerId}:`, err);
      onError?.(t("modelSelector.switchProviderError"));
    }
  };

  React.useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setShowDropdown(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const label = selectedProvider
    ? getTranslatedModelName(selectedProvider, t)
    : t("modelSelector.notAvailable");

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        type="button"
        onClick={() => setShowDropdown((value) => !value)}
        className="flex items-center gap-2 rounded-lg border border-glass-border bg-glass-bg px-3 py-2 text-sm text-text/80 transition-colors hover:bg-glass-highlight"
      >
        <Cloud weight="fill" className="h-3.5 w-3.5 text-text/40" />
        <span className="max-w-36 truncate">{label}</span>
        <CaretDown
          className={`h-3.5 w-3.5 text-text/40 transition-transform ${showDropdown ? "rotate-180" : ""}`}
        />
      </button>

      {showDropdown && (
        <ModelDropdown
          providers={providers}
          sttProviderId={sttProviderId}
          onCloudProviderSelect={handleProviderSelect}
        />
      )}
    </div>
  );
};

export default ModelSelector;
