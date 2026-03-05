import React from "react";
import * as SelectPrimitive from "@radix-ui/react-select";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

export interface DropdownOption {
  value: string;
  label: string;
  disabled?: boolean;
}

interface DropdownProps {
  options: DropdownOption[];
  className?: string;
  selectedValue: string | null;
  onSelect: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  onRefresh?: () => void;
}

export const Dropdown: React.FC<DropdownProps> = ({
  options,
  selectedValue,
  onSelect,
  className = "",
  placeholder = "Select an option...",
  disabled = false,
  onRefresh,
}) => {
  const { t } = useTranslation();

  return (
    <SelectPrimitive.Root
      value={selectedValue ?? undefined}
      onValueChange={onSelect}
      disabled={disabled}
      onOpenChange={(open) => {
        if (open && onRefresh) onRefresh();
      }}
    >
      <SelectPrimitive.Trigger
        className={cn(
          "px-2 py-1 text-sm font-semibold bg-glass-bg border border-glass-border rounded backdrop-blur-sm",
          "min-w-[160px] w-full text-start flex items-center justify-between",
          "transition-all duration-150",
          disabled
            ? "opacity-50 cursor-not-allowed"
            : "hover:bg-glass-highlight cursor-pointer hover:border-glass-border-hover",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
          className,
        )}
      >
        <span className="truncate">
          <SelectPrimitive.Value placeholder={placeholder} />
        </span>
        <SelectPrimitive.Icon>
          <svg
            className="w-4 h-4 ms-2"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>

      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          className={cn(
            "overflow-hidden glass-panel rounded-lg shadow-glass-hover z-50",
            "min-w-[var(--radix-select-trigger-width)]",
            "animate-in fade-in-0 zoom-in-95",
            "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
            "data-[side=bottom]:slide-in-from-top-2 data-[side=top]:slide-in-from-bottom-2",
          )}
          position="popper"
          sideOffset={4}
          align="start"
        >
          <SelectPrimitive.Viewport className="max-h-60">
            {options.length === 0 ? (
              <div className="px-2 py-1 text-sm text-muted-foreground">
                {t("common.noOptionsFound")}
              </div>
            ) : (
              options.map((option) => (
                <SelectPrimitive.Item
                  key={option.value}
                  value={option.value}
                  disabled={option.disabled}
                  className={cn(
                    "w-full px-2 py-1 text-sm text-start cursor-pointer select-none outline-none",
                    "transition-colors duration-150",
                    "data-[highlighted]:bg-primary/10",
                    "data-[state=checked]:bg-primary/20 data-[state=checked]:font-semibold",
                    "data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed",
                  )}
                >
                  <SelectPrimitive.ItemText>
                    {option.label}
                  </SelectPrimitive.ItemText>
                </SelectPrimitive.Item>
              ))
            )}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
};
