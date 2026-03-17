import React from "react";
import { ResetButton } from "../ui/ResetButton";

interface ShortcutKeyBadgeProps {
  isEditing: boolean;
  editingRef: React.Ref<HTMLDivElement>;
  currentKeysDisplay: string;
  bindingDisplay: string;
  onStartRecording: () => void;
  onReset: () => void;
  resetDisabled: boolean;
}

export const ShortcutKeyBadge: React.FC<ShortcutKeyBadgeProps> = ({
  isEditing,
  editingRef,
  currentKeysDisplay,
  bindingDisplay,
  onStartRecording,
  onReset,
  resetDisabled,
}) => (
  <div className="flex items-center space-x-1">
    {isEditing ? (
      <div
        ref={editingRef}
        className="px-2 py-1 text-sm font-semibold border border-accent bg-accent/30 rounded"
      >
        {currentKeysDisplay}
      </div>
    ) : (
      <button
        type="button"
        className="px-2 py-1 text-sm font-semibold bg-muted/10 border border-muted/80 hover:bg-accent/10 rounded cursor-pointer hover:border-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        onClick={onStartRecording}
      >
        {bindingDisplay}
      </button>
    )}
    <ResetButton onClick={onReset} disabled={resetDisabled} />
  </div>
);
