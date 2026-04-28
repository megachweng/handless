import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import type { OverlayPosition } from "@/bindings";
import { cn } from "@/lib/utils";

interface ShowOverlayProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ShowOverlay: React.FC<ShowOverlayProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const overlayOptions = useMemo(() => {
      return [
        {
          value: "bottom",
          label: t("settings.advanced.overlay.options.bottom"),
        },
        { value: "top", label: t("settings.advanced.overlay.options.top") },
      ];
    }, [t]);

    const selectedPosition = (getSetting("overlay_position") ||
      "bottom") as OverlayPosition;
    const overlayEnabled = getSetting("overlay_enabled");
    const isEnabled = overlayEnabled ?? selectedPosition !== "none";
    const effectivePosition =
      selectedPosition === "none" ? "bottom" : selectedPosition;

    const isOverlayUpdating =
      isUpdating("overlay_enabled") || isUpdating("overlay_position");

    return (
      <>
        <ToggleSwitch
          label={t("settings.advanced.overlay.title")}
          description={t("settings.advanced.overlay.description")}
          checked={isEnabled}
          onChange={async (checked) => {
            if (checked && selectedPosition === "none") {
              await updateSetting("overlay_position", "bottom");
            }

            await updateSetting("overlay_enabled", checked);
          }}
          disabled={isOverlayUpdating}
          grouped={grouped}
          descriptionMode={descriptionMode}
        />
        {isEnabled && (
          <SettingContainer
            title={t("settings.advanced.overlay.position_label")}
            description=""
            grouped={grouped}
            descriptionMode="tooltip"
          >
            <div className="flex bg-glass-bg border border-glass-border rounded-lg p-0.5 gap-0.5">
              {overlayOptions.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  onClick={() =>
                    updateSetting(
                      "overlay_position",
                      opt.value as OverlayPosition,
                    )
                  }
                  disabled={isOverlayUpdating}
                  className={cn(
                    "px-3 py-1.5 text-xs rounded-md transition-all duration-200 cursor-pointer select-none",
                    effectivePosition === opt.value
                      ? "bg-accent text-accent-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-glass-highlight hover:text-foreground",
                  )}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </SettingContainer>
        )}
      </>
    );
  },
);
