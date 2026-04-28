import React from "react";
import { Check, Cloud } from "@phosphor-icons/react";
import type { SttProviderInfo } from "@/bindings";
import { getTranslatedModelName } from "../../lib/utils/modelTranslation";
import { useTranslation } from "react-i18next";

interface ModelDropdownProps {
  providers: SttProviderInfo[];
  sttProviderId: string;
  onCloudProviderSelect: (providerId: string) => void;
}

const ModelDropdown: React.FC<ModelDropdownProps> = ({
  providers,
  sttProviderId,
  onCloudProviderSelect,
}) => {
  const { t } = useTranslation();

  return (
    <div className="absolute bottom-full -start-4 z-50 mb-2 max-h-[60vh] w-56 origin-bottom-left overflow-y-auto overscroll-contain rounded-xl border border-glass-border bg-[var(--color-glass-bg-solid)] p-1.5 shadow-glass-hover animate-in fade-in-0 slide-in-from-bottom-1 zoom-in-[0.97] duration-150">
      {providers.length > 0 ? (
        providers.map((provider) => {
          const active = sttProviderId === provider.id;
          return (
            <button
              key={provider.id}
              type="button"
              onClick={() => onCloudProviderSelect(provider.id)}
              className={`w-full cursor-pointer rounded-md px-3 py-2 text-start transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
                active
                  ? "bg-accent/10 text-text"
                  : "text-text/70 hover:bg-glass-highlight hover:text-text"
              }`}
            >
              <div className="flex items-center justify-between gap-2">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-1.5 text-[13px] leading-tight">
                    {getTranslatedModelName(provider, t)}
                    <Cloud weight="fill" className="h-3 w-3 text-text/20" />
                  </div>
                  <div className="mt-0.5 text-xs leading-snug text-text/30">
                    {t(provider.description)}
                  </div>
                </div>
                {active && (
                  <Check
                    weight="bold"
                    className="h-3.5 w-3.5 shrink-0 text-accent"
                  />
                )}
              </div>
            </button>
          );
        })
      ) : (
        <div className="px-3 py-3 text-center text-[13px] text-text/40">
          {t("modelSelector.noModelsAvailable")}
        </div>
      )}
    </div>
  );
};

export default ModelDropdown;
