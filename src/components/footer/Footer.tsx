import React, { useMemo } from "react";
import { useAppVersion } from "@/hooks/useAppVersion";
import { useModelStore } from "@/stores/modelStore";

import ModelSelector from "../model-selector";
import UpdateChecker from "../update-checker";

const Footer: React.FC = () => {
  const version = useAppVersion();
  const downloadProgress = useModelStore((s) => s.downloadProgress);
  const extractingModels = useModelStore((s) => s.extractingModels);

  const overallProgress = useMemo(() => {
    const progressValues = Object.values(downloadProgress);
    if (progressValues.length === 0) return null;
    const total = progressValues.reduce((sum, p) => sum + p.percentage, 0);
    return total / progressValues.length;
  }, [downloadProgress]);

  const isExtracting = Object.keys(extractingModels).length > 0;

  return (
    <div className="w-full border-t border-glass-border bg-glass-bg pt-2 relative">
      {/* Thin progress bar along top edge during downloads */}
      {overallProgress !== null && (
        <div className="absolute top-0 inset-x-0 h-[2px] bg-muted/10">
          <div
            className="h-full bg-accent transition-[width] duration-300 ease-out"
            style={{ width: `${Math.max(0, Math.min(100, overallProgress))}%` }}
          />
        </div>
      )}
      {isExtracting && overallProgress === null && (
        <div className="absolute top-0 inset-x-0 h-[2px] bg-accent/60 animate-pulse" />
      )}
      <div className="flex justify-between items-center text-xs px-4 pb-2 text-text/70">
        <div className="flex items-center gap-4">
          <ModelSelector />
        </div>

        {/* Update Status */}
        <div className="flex items-center gap-1">
          <UpdateChecker />
          <span>•</span>
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span>v{version}</span>
        </div>
      </div>
    </div>
  );
};

export default Footer;
