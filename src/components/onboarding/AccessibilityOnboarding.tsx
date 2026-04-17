import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { motion, AnimatePresence } from "motion/react";
import {
  Keyboard,
  Microphone,
  Check,
  CircleNotch,
} from "@phosphor-icons/react";
import { spring, tapScale } from "@/lib/motion";
import { DragRegion } from "@/components/ui/DragRegion";
import {
  type PermissionKind,
  type PermissionStatus,
  usePermissionRequests,
} from "@/hooks/usePermissionRequests";

interface AccessibilityOnboardingProps {
  onComplete: () => void;
}

const PermissionRow: React.FC<{
  icon: React.ReactNode;
  label: string;
  status: PermissionStatus;
  onGrant: () => void;
}> = ({ icon, label, status, onGrant }) => {
  const { t } = useTranslation();

  return (
    <div className="flex items-center justify-between gap-4 py-3.5">
      <div className="flex items-center gap-3">
        <span className="text-text/50">{icon}</span>
        <span className="text-sm font-medium text-text/80">{label}</span>
      </div>
      <AnimatePresence mode="wait">
        {status === "granted" ? (
          <motion.span
            key="granted"
            className="flex items-center gap-1.5 text-xs text-success/80"
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={spring.stiff}
          >
            <Check weight="bold" className="h-3.5 w-3.5" />
            {t("onboarding.permissions.granted")}
          </motion.span>
        ) : status === "waiting" ? (
          <motion.span
            key="waiting"
            className="flex items-center gap-1.5 text-xs text-text/30"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <CircleNotch className="h-3.5 w-3.5 animate-spin" />
            {t("onboarding.permissions.waiting")}
          </motion.span>
        ) : status === "needed" ? (
          <motion.button
            key="grant"
            onClick={onGrant}
            className="cursor-pointer text-xs text-accent transition-colors hover:text-accent/80"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            whileTap={tapScale}
          >
            {t("onboarding.permissions.grant")}
          </motion.button>
        ) : null}
      </AnimatePresence>
    </div>
  );
};

const AccessibilityOnboarding: React.FC<AccessibilityOnboardingProps> = ({
  onComplete,
}) => {
  const { t } = useTranslation();
  const { allGranted, isMacOS, permissions, requestPermission } =
    usePermissionRequests();

  useEffect(() => {
    if (!isMacOS) {
      onComplete();
      return;
    }

    if (!allGranted) {
      return;
    }

    const timeoutId = window.setTimeout(() => onComplete(), 500);
    return () => window.clearTimeout(timeoutId);
  }, [allGranted, isMacOS, onComplete]);

  const handleGrant = async (kind: PermissionKind) => {
    try {
      await requestPermission(kind);
    } catch (error) {
      console.error(`Failed to request ${kind} permission:`, error);
      toast.error(t("onboarding.permissions.errors.requestFailed"));
    }
  };

  const isChecking =
    permissions.accessibility === "checking" &&
    permissions.microphone === "checking";

  const content = (() => {
    if (isMacOS && isChecking) {
      return (
        <div className="flex flex-1 items-center justify-center">
          <CircleNotch className="h-5 w-5 animate-spin text-text/20" />
        </div>
      );
    }

    if (allGranted) {
      return (
        <div className="flex flex-1 flex-col items-center justify-center gap-3">
          <motion.div
            initial={{ scale: 0, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={spring.stiff}
          >
            <Check weight="bold" className="h-8 w-8 text-success/70" />
          </motion.div>
          <motion.p
            className="text-sm text-text/50"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.15 }}
          >
            {t("onboarding.permissions.allGranted")}
          </motion.p>
        </div>
      );
    }

    return (
      <div className="flex flex-1 flex-col items-center justify-center px-8">
        <motion.div
          className="flex w-full max-w-xs flex-col items-center"
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, ease: [0.25, 0.1, 0.25, 1] }}
        >
          <h1 className="mb-1 text-3xl font-bold tracking-tight text-text">
            {t("appName")}
          </h1>
          <p className="mb-10 text-xs text-text/40">
            {t("onboarding.permissions.description")}
          </p>

          <div className="w-full">
            <PermissionRow
              icon={<Microphone weight="regular" className="h-4 w-4" />}
              label={t("onboarding.permissions.microphone.title")}
              status={permissions.microphone}
              onGrant={() => {
                void handleGrant("microphone");
              }}
            />
            <div className="h-px bg-text/[0.04]" />
            <PermissionRow
              icon={<Keyboard weight="regular" className="h-4 w-4" />}
              label={t("onboarding.permissions.accessibility.title")}
              status={permissions.accessibility}
              onGrant={() => {
                void handleGrant("accessibility");
              }}
            />
          </div>
        </motion.div>
      </div>
    );
  })();

  return (
    <div className="flex h-screen w-screen flex-col">
      <DragRegion />
      {content}
    </div>
  );
};

export default AccessibilityOnboarding;
