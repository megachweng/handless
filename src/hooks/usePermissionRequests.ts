export type PermissionKind = "accessibility" | "microphone";
export type PermissionStatus = "checking" | "needed" | "waiting" | "granted";

export interface PermissionsState {
  accessibility: PermissionStatus;
  microphone: PermissionStatus;
}

const GRANTED_STATE: PermissionsState = {
  accessibility: "granted",
  microphone: "granted",
};

export function usePermissionRequests() {
  return {
    allGranted: true,
    isMacOS: false,
    permissions: GRANTED_STATE,
    requestPermission: async (_kind: PermissionKind) => {},
  };
}
