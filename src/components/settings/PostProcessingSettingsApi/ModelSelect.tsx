import React from "react";
import { Dropdown, type DropdownOption } from "../../ui/Dropdown";

type ModelSelectProps = {
  value: string;
  options: DropdownOption[];
  disabled?: boolean;
  placeholder?: string;
  isLoading?: boolean;
  onSelect: (value: string) => void;
  onCreateValue?: (value: string) => void;
  className?: string;
};

export const ModelSelect: React.FC<ModelSelectProps> = React.memo(
  ({
    value,
    options,
    disabled,
    placeholder,
    isLoading,
    onSelect,
    onCreateValue,
    className = "flex-1",
  }) => {
    return (
      <Dropdown
        selectedValue={value || null}
        options={options}
        onSelect={onSelect}
        placeholder={placeholder}
        disabled={disabled || isLoading}
        className={className}
        creatable
        onCreateValue={onCreateValue}
      />
    );
  },
);

ModelSelect.displayName = "ModelSelect";
