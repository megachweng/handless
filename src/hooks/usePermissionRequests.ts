import { useCallback, useEffect, useRef, useState } from "react";
import { platform } from "@tauri-apps/plugin-os";
import {
  checkAccessibilityPermission,
  checkMicrophonePermission,
  requestMicrophonePermission,
} from "tauri-plugin-macos-permissions-api";
import { commands, type PermissionAssistantPanel } from "@/bindings";

export type PermissionKind = "accessibility" | "microphone";
export type PermissionStatus = "checking" | "needed" | "waiting" | "granted";

export interface PermissionsState {
  accessibility: PermissionStatus;
  microphone: PermissionStatus;
}

const INITIAL_STATE: PermissionsState = {
  accessibility: "checking",
  microphone: "checking",
};

const POLL_INTERVAL_MS = 1000;
const MAX_POLLING_ERRORS = 3;
const WAITING_RESET_MS = 3000;

async function ensureCommandOk<T>(
  promise: Promise<
    { status: "ok"; data: T } | { status: "error"; error: string }
  >,
): Promise<T> {
  const result = await promise;
  if (result.status === "error") {
    throw new Error(result.error);
  }

  return result.data;
}

export function usePermissionRequests() {
  const isMacOS = platform() === "macos";
  const [permissions, setPermissions] =
    useState<PermissionsState>(INITIAL_STATE);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const waitingResetRef = useRef<
    Record<PermissionKind, ReturnType<typeof setTimeout> | null>
  >({
    accessibility: null,
    microphone: null,
  });
  const errorCountRef = useRef(0);
  const activeAssistantRef = useRef<PermissionAssistantPanel | null>(null);

  const clearWaitingReset = useCallback((kind: PermissionKind) => {
    const timeoutId = waitingResetRef.current[kind];
    if (timeoutId) {
      clearTimeout(timeoutId);
      waitingResetRef.current[kind] = null;
    }
  }, []);

  const scheduleWaitingReset = useCallback(
    (kind: PermissionKind) => {
      clearWaitingReset(kind);
      waitingResetRef.current[kind] = setTimeout(() => {
        waitingResetRef.current[kind] = null;
        setPermissions((prev) =>
          prev[kind] === "waiting" ? { ...prev, [kind]: "needed" } : prev,
        );
      }, WAITING_RESET_MS);
    },
    [clearWaitingReset],
  );

  const dismissAssistant = useCallback(async () => {
    if (!activeAssistantRef.current) {
      return;
    }

    activeAssistantRef.current = null;
    try {
      await ensureCommandOk(commands.dismissPermissionAssistant());
    } catch (error) {
      console.warn("Failed to dismiss permission assistant:", error);
    }
  }, []);

  const stopPolling = useCallback(() => {
    if (pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
    errorCountRef.current = 0;
  }, []);

  const refreshPermissions = useCallback(async () => {
    if (!isMacOS) {
      setPermissions({
        accessibility: "granted",
        microphone: "granted",
      });
      return {
        accessibility: true,
        microphone: true,
      };
    }

    const [accessibilityGranted, microphoneGranted] = await Promise.all([
      checkAccessibilityPermission(),
      checkMicrophonePermission(),
    ]);

    if (accessibilityGranted) {
      clearWaitingReset("accessibility");
    }

    if (microphoneGranted) {
      clearWaitingReset("microphone");
    }

    setPermissions((prev) => ({
      accessibility: accessibilityGranted
        ? "granted"
        : prev.accessibility === "waiting"
          ? "waiting"
          : "needed",
      microphone: microphoneGranted
        ? "granted"
        : prev.microphone === "waiting"
          ? "waiting"
          : "needed",
    }));

    return {
      accessibility: accessibilityGranted,
      microphone: microphoneGranted,
    };
  }, [clearWaitingReset, isMacOS]);

  const pollOnce = useCallback(async () => {
    try {
      const nextState = await refreshPermissions();
      errorCountRef.current = 0;

      if (
        nextState.accessibility &&
        activeAssistantRef.current === "accessibility"
      ) {
        await dismissAssistant();
      }

      if (nextState.microphone && activeAssistantRef.current === "microphone") {
        await dismissAssistant();
      }

      if (nextState.accessibility && nextState.microphone) {
        stopPolling();
      }
    } catch (error) {
      console.error("Error checking permissions:", error);
      errorCountRef.current += 1;

      if (errorCountRef.current >= MAX_POLLING_ERRORS) {
        stopPolling();
      }
    }
  }, [dismissAssistant, refreshPermissions, stopPolling]);

  const startPolling = useCallback(() => {
    if (pollingRef.current || !isMacOS) {
      return;
    }

    pollingRef.current = setInterval(() => {
      void pollOnce();
    }, POLL_INTERVAL_MS);
  }, [isMacOS, pollOnce]);

  const presentAssistant = useCallback(
    async (panel: PermissionAssistantPanel) => {
      await ensureCommandOk(commands.presentPermissionAssistant(panel));
      activeAssistantRef.current = panel;
    },
    [],
  );

  const requestPermission = useCallback(
    async (kind: PermissionKind) => {
      if (!isMacOS) {
        return;
      }

      if (kind === "accessibility") {
        await presentAssistant("accessibility");
        setPermissions((prev) => ({ ...prev, accessibility: "waiting" }));
        scheduleWaitingReset("accessibility");
        startPolling();
        return;
      }

      setPermissions((prev) => ({ ...prev, microphone: "waiting" }));
      scheduleWaitingReset("microphone");

      try {
        await requestMicrophonePermission();
      } catch (error) {
        console.warn("Microphone permission request did not complete:", error);
      }

      const microphoneGranted = await checkMicrophonePermission();
      if (microphoneGranted) {
        clearWaitingReset("microphone");
        setPermissions((prev) => ({ ...prev, microphone: "granted" }));
        return;
      }

      await presentAssistant("microphone");
      startPolling();
    },
    [
      clearWaitingReset,
      isMacOS,
      presentAssistant,
      scheduleWaitingReset,
      startPolling,
    ],
  );

  useEffect(() => {
    void refreshPermissions().catch((error) => {
      console.error("Failed to check permissions:", error);
      setPermissions({
        accessibility: "needed",
        microphone: "needed",
      });
    });
  }, [refreshPermissions]);

  useEffect(() => {
    return () => {
      stopPolling();
      clearWaitingReset("accessibility");
      clearWaitingReset("microphone");
      void dismissAssistant();
    };
  }, [clearWaitingReset, dismissAssistant, stopPolling]);

  return {
    isMacOS,
    permissions,
    allGranted:
      permissions.accessibility === "granted" &&
      permissions.microphone === "granted",
    requestPermission,
    refreshPermissions,
  };
}
