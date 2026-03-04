import React from "react";
import { cn } from "@/lib/utils";
import { Slider } from "./Slider";
import { SettingContainer } from "./SettingContainer";

interface SliderSettingProps {
  value: number;
  onChange: (value: number) => void;
  min: number;
  max: number;
  step?: number;
  disabled?: boolean;
  label: string;
  description: string;
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  showValue?: boolean;
  formatValue?: (value: number) => string;
}

export const SliderSetting: React.FC<SliderSettingProps> = ({
  value,
  onChange,
  min,
  max,
  step = 0.01,
  disabled = false,
  label,
  description,
  descriptionMode = "tooltip",
  grouped = false,
  showValue = true,
  formatValue = (v) => v.toFixed(2),
}) => {
  return (
    <SettingContainer
      title={label}
      description={description}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
      disabled={disabled}
    >
      <div className="w-40">
        <div className="flex items-center space-x-1 h-6">
          <Slider
            className={cn(
              "flex-grow",
              disabled && "opacity-50 pointer-events-none",
            )}
            value={[value]}
            onValueChange={([v]) => onChange(v)}
            min={min}
            max={max}
            step={step}
            disabled={disabled}
          />
          {showValue && (
            <span className="text-sm font-medium text-text/90 min-w-10 text-end">
              {formatValue(value)}
            </span>
          )}
        </div>
      </div>
    </SettingContainer>
  );
};
