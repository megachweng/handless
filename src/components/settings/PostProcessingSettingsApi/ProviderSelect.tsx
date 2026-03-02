import React from "react";
import { Dropdown, type DropdownOption } from "../../ui/Dropdown";

interface ProviderSelectProps {
  options: DropdownOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  className?: string;
}

export const ProviderSelect: React.FC<ProviderSelectProps> = React.memo(
  ({ options, value, onChange, disabled, className = "flex-1" }) => {
    return (
      <Dropdown
        options={options}
        selectedValue={value}
        onSelect={onChange}
        disabled={disabled}
        className={className}
      />
    );
  },
);

ProviderSelect.displayName = "ProviderSelect";
