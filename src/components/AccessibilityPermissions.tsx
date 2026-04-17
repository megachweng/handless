import { useTranslation } from "react-i18next";
import { CircleNotch, Keyboard, Microphone } from "@phosphor-icons/react";
import {
  type PermissionKind,
  type PermissionStatus,
  usePermissionRequests,
} from "@/hooks/usePermissionRequests";

const CompactPermissionRow: React.FC<{
  icon: React.ReactNode;
  label: string;
  description: string;
  status: PermissionStatus;
  onGrant: () => void;
}> = ({ icon, label, description, status, onGrant }) => {
  const { t } = useTranslation();

  return (
    <div className="flex items-start justify-between gap-4 py-3">
      <div className="flex items-start gap-3">
        <span className="mt-0.5 text-text/50">{icon}</span>
        <div className="space-y-1">
          <p className="text-sm font-medium text-text/85">{label}</p>
          <p className="text-xs text-text/50">{description}</p>
        </div>
      </div>

      {status === "waiting" ? (
        <span className="flex items-center gap-1.5 text-xs text-text/35">
          <CircleNotch className="h-3.5 w-3.5 animate-spin" />
          {t("onboarding.permissions.waiting")}
        </span>
      ) : (
        <button
          onClick={onGrant}
          className="min-h-9 cursor-pointer rounded-lg border border-glass-border bg-glass-bg px-3 text-xs font-medium text-text transition-colors hover:bg-glass-highlight"
        >
          {t("onboarding.permissions.grant")}
        </button>
      )}
    </div>
  );
};

const AccessibilityPermissions: React.FC = () => {
  const { t } = useTranslation();
  const { allGranted, isMacOS, permissions, requestPermission } =
    usePermissionRequests();

  if (!isMacOS || allGranted) {
    return null;
  }

  const handleGrant = async (kind: PermissionKind) => {
    try {
      await requestPermission(kind);
    } catch (error) {
      console.error(`Error requesting ${kind} permission:`, error);
    }
  };

  return (
    <div className="w-full rounded-xl border border-glass-border bg-glass-bg/80 p-4 shadow-glass">
      <div className="space-y-1">
        <p className="text-sm font-medium text-text/90">
          {t("accessibility.permissionsRequired")}
        </p>
        <p className="text-xs text-text/50">
          {t("onboarding.permissions.description")}
        </p>
      </div>

      <div className="mt-3 divide-y divide-white/5">
        {permissions.microphone !== "granted" && (
          <CompactPermissionRow
            icon={<Microphone className="h-4 w-4" weight="regular" />}
            label={t("onboarding.permissions.microphone.title")}
            description={t("onboarding.permissions.microphone.description")}
            status={permissions.microphone}
            onGrant={() => {
              void handleGrant("microphone");
            }}
          />
        )}

        {permissions.accessibility !== "granted" && (
          <CompactPermissionRow
            icon={<Keyboard className="h-4 w-4" weight="regular" />}
            label={t("onboarding.permissions.accessibility.title")}
            description={t("onboarding.permissions.accessibility.description")}
            status={permissions.accessibility}
            onGrant={() => {
              void handleGrant("accessibility");
            }}
          />
        )}
      </div>
    </div>
  );
};

export default AccessibilityPermissions;
