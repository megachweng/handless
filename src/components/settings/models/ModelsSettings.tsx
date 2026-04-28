import React from "react";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { useModelStore } from "@/stores/modelStore";
import { staggerContainer, staggerItem } from "@/lib/motion";
import { CloudProviderConfigCard } from "./CloudProviderConfigCard";
import { useSettings } from "@/hooks/useSettings";

const EMPTY_ARRAY: string[] = [];

export const ModelsSettings: React.FC = () => {
  const { t } = useTranslation();
  const { providers, loading } = useModelStore();
  const {
    settings,
    setSttProvider,
    updateSttApiKey,
    updateSttCloudModel,
    updateSttCloudOptions,
    updateSttRealtimeEnabled,
    verifySttProvider,
    isUpdating,
  } = useSettings();

  const sttProviderId = settings?.stt_provider_id ?? "soniox";
  const verifiedProviders = settings?.stt_verified_providers ?? EMPTY_ARRAY;

  return (
    <motion.div
      className="w-full max-w-3xl space-y-8"
      variants={staggerContainer}
      initial="initial"
      animate="animate"
    >
      <motion.div
        className="mb-4"
        variants={staggerItem}
        style={{ willChange: "transform" }}
      >
        <h1 className="mb-2 text-xl font-semibold">
          {t("settings.models.title")}
        </h1>
        <p className="text-sm text-muted-foreground">
          {t("settings.models.description")}
        </p>
      </motion.div>

      <motion.div
        className="space-y-2"
        variants={staggerItem}
        style={{ willChange: "transform" }}
      >
        {loading ? (
          <div className="rounded border border-glass-border bg-background-translucent">
            <div className="flex flex-col items-center gap-3 px-3 py-8">
              <div className="h-5 w-5 animate-spin rounded-full border-2 border-muted/40 border-t-accent" />
            </div>
          </div>
        ) : (
          providers.map((provider) => (
            <CloudProviderConfigCard
              key={provider.id}
              provider={provider}
              compact
              status={sttProviderId === provider.id ? "active" : "available"}
              onSelect={setSttProvider}
              apiKey={settings?.stt_api_keys?.[provider.id] ?? ""}
              cloudModel={
                settings?.stt_cloud_models?.[provider.id] ??
                provider.backend.default_model
              }
              onApiKeyChange={(apiKey) => updateSttApiKey(provider.id, apiKey)}
              onModelChange={(model) => updateSttCloudModel(provider.id, model)}
              onVerify={verifySttProvider}
              isVerifying={isUpdating(`stt_verify:${provider.id}`)}
              isVerified={verifiedProviders.includes(provider.id)}
              cloudOptions={
                settings?.stt_cloud_options?.[provider.id]
                  ? JSON.parse(settings.stt_cloud_options[provider.id]!)
                  : {}
              }
              onOptionsChange={(opts) =>
                updateSttCloudOptions(provider.id, opts)
              }
              realtimeEnabled={
                settings?.stt_realtime_enabled?.[provider.id] ?? false
              }
              onRealtimeChange={(enabled) =>
                updateSttRealtimeEnabled(provider.id, enabled)
              }
              dictionaryTerms={settings?.dictionary_terms ?? []}
              dictionaryContext={settings?.dictionary_context ?? ""}
            />
          ))
        )}
      </motion.div>
    </motion.div>
  );
};
