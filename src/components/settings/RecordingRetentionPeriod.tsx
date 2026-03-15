import React, { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion, AnimatePresence } from "motion/react";
import { Dropdown } from "../ui/Dropdown";
import { NumberInput } from "../ui/NumberInput";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import { commands, RecordingRetentionPeriod } from "@/bindings";
import { spring, tapScale } from "@/lib/motion";

interface RecordingRetentionPeriodProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const RecordingRetentionPeriodSelector: React.FC<RecordingRetentionPeriodProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const currentPeriod = getSetting("recording_retention_period") || "never";
    const historyLimit = getSetting("history_limit") || 5;

    const [pendingPeriod, setPendingPeriod] = useState<string | null>(null);
    const [pendingLimit, setPendingLimit] = useState<number | null>(null);
    const [affectedCount, setAffectedCount] = useState<number | null>(null);
    const [previewing, setPreviewing] = useState(false);
    const previewTimerRef = useRef<ReturnType<typeof setTimeout>>(null);

    const retentionOptions = [
      { value: "never", label: t("settings.debug.recordingRetention.never") },
      {
        value: "preserve_limit",
        label: t("settings.debug.recordingRetention.preserveLimit"),
      },
      { value: "days3", label: t("settings.debug.recordingRetention.days3") },
      { value: "weeks2", label: t("settings.debug.recordingRetention.weeks2") },
      {
        value: "months3",
        label: t("settings.debug.recordingRetention.months3"),
      },
    ];

    const cancelPending = () => {
      setPendingPeriod(null);
      setPendingLimit(null);
      setAffectedCount(null);
    };

    const previewCleanup = async (period: string, limit: number | null) => {
      setPreviewing(true);
      try {
        const result = await commands.previewRetentionCleanup(period, limit);
        if (result.status === "ok") {
          setAffectedCount(result.data);
        }
      } catch {
        setAffectedCount(null);
      } finally {
        setPreviewing(false);
      }
    };

    const handleRetentionPeriodSelect = async (period: string) => {
      if (period === "never") {
        cancelPending();
        await updateSetting(
          "recording_retention_period",
          period as RecordingRetentionPeriod,
        );
        return;
      }

      setPendingPeriod(period);
      setPendingLimit(null);
      await previewCleanup(
        period,
        period === "preserve_limit" ? historyLimit : null,
      );
    };

    const handleLimitChange = (value: number | undefined) => {
      if (value === undefined) return;

      if (pendingPeriod !== "preserve_limit" && currentPeriod === "preserve_limit") {
        setPendingPeriod("preserve_limit");
      }
      setPendingLimit(value);

      if (previewTimerRef.current) clearTimeout(previewTimerRef.current);
      previewTimerRef.current = setTimeout(() => {
        previewCleanup("preserve_limit", value);
      }, 300);
    };

    const handleConfirm = async () => {
      if (!pendingPeriod) return;
      const period = pendingPeriod;
      const limit = pendingLimit;
      cancelPending();

      if (limit !== null) {
        await updateSetting("history_limit", limit);
      }
      await updateSetting(
        "recording_retention_period",
        period as RecordingRetentionPeriod,
      );
    };

    const showConfirmation = pendingPeriod !== null && affectedCount !== null;
    const displayPeriod = pendingPeriod ?? currentPeriod;
    const displayLimit = pendingLimit ?? historyLimit;
    const isDestructive = showConfirmation && affectedCount > 0;

    return (
      <div>
        <SettingContainer
          title={t("settings.debug.recordingRetention.title")}
          description={t("settings.debug.recordingRetention.description")}
          descriptionMode={descriptionMode}
          grouped={grouped}
        >
          <div className="flex items-center gap-2">
            <Dropdown
              options={retentionOptions}
              selectedValue={displayPeriod}
              onSelect={handleRetentionPeriodSelect}
              placeholder={t("settings.debug.recordingRetention.placeholder")}
              disabled={isUpdating("recording_retention_period") || previewing}
            />
            {displayPeriod === "preserve_limit" && (
              <NumberInput
                min={1}
                max={1000}
                value={displayLimit}
                onChange={handleLimitChange}
                disabled={isUpdating("history_limit") || previewing}
              />
            )}
          </div>
        </SettingContainer>

        <AnimatePresence>
          {showConfirmation && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
              className="overflow-hidden"
            >
              <div className="pt-3 px-3">
                <div
                  className={`rounded-lg border px-3.5 py-3 flex items-center justify-between gap-4 ${
                    isDestructive
                      ? "border-error/20 bg-error/[0.04]"
                      : "border-glass-border bg-glass-bg"
                  }`}
                >
                  <p
                    className={`text-xs leading-relaxed ${
                      isDestructive ? "text-text/70" : "text-muted"
                    }`}
                  >
                    {isDestructive
                      ? t(
                          "settings.debug.recordingRetention.confirmDelete",
                          { count: affectedCount },
                        )
                      : t(
                          "settings.debug.recordingRetention.noEntriesAffected",
                        )}
                  </p>
                  <div className="flex items-center gap-1.5 shrink-0">
                    <motion.button
                      onClick={cancelPending}
                      whileTap={tapScale}
                      transition={spring.snappy}
                      className="px-2.5 py-1 text-xs text-muted hover:text-text transition-colors rounded-md cursor-pointer"
                    >
                      {t("settings.debug.recordingRetention.confirmCancel")}
                    </motion.button>
                    <motion.button
                      onClick={handleConfirm}
                      whileTap={tapScale}
                      transition={spring.snappy}
                      className={`px-2.5 py-1 text-xs font-medium rounded-md transition-colors cursor-pointer ${
                        isDestructive
                          ? "text-error hover:bg-error/10"
                          : "text-accent hover:bg-accent/10"
                      }`}
                    >
                      {isDestructive
                        ? t(
                            "settings.debug.recordingRetention.confirmApply",
                          )
                        : t("common.save")}
                    </motion.button>
                  </div>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    );
  });

RecordingRetentionPeriodSelector.displayName =
  "RecordingRetentionPeriodSelector";
