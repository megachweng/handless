import React from "react";
import SelectComponent from "react-select";
import CreatableSelect from "react-select/creatable";
import type {
  ActionMeta,
  Props as ReactSelectProps,
  SingleValue,
  StylesConfig,
} from "react-select";

export type SelectOption = {
  value: string;
  label: string;
  isDisabled?: boolean;
};

type BaseProps = {
  value: string | null;
  options: SelectOption[];
  placeholder?: string;
  disabled?: boolean;
  isLoading?: boolean;
  isClearable?: boolean;
  onChange: (value: string | null, action: ActionMeta<SelectOption>) => void;
  onBlur?: () => void;
  className?: string;
  formatCreateLabel?: (input: string) => string;
};

type CreatableProps = {
  isCreatable: true;
  onCreateOption: (value: string) => void;
};

type NonCreatableProps = {
  isCreatable?: false;
  onCreateOption?: never;
};

export type SelectProps = BaseProps & (CreatableProps | NonCreatableProps);

const glassBg = "var(--color-glass-bg)";
const glassBorder = "var(--color-glass-border)";
const glassBorderHover = "var(--color-glass-border-hover)";
const hoverBackground =
  "color-mix(in srgb, var(--color-accent) 12%, transparent)";
const focusBackground =
  "color-mix(in srgb, var(--color-accent) 20%, transparent)";

const selectStyles: StylesConfig<SelectOption, false> = {
  control: (base, state) => ({
    ...base,
    minHeight: 30,
    borderRadius: 8,
    borderColor: state.isFocused ? "var(--color-accent)" : glassBorder,
    boxShadow: state.isFocused
      ? "0 0 0 1px var(--color-accent), var(--shadow-accent-glow)"
      : "var(--shadow-glass)",
    backgroundColor: state.isFocused ? focusBackground : glassBg,
    backdropFilter: "blur(var(--glass-blur-light))",
    fontSize: "0.875rem",
    color: "var(--color-text)",
    transition: "all 150ms ease",
    ":hover": {
      borderColor: glassBorderHover,
      backgroundColor: hoverBackground,
    },
  }),
  valueContainer: (base) => ({
    ...base,
    paddingInline: 8,
    paddingBlock: 2,
  }),
  input: (base) => ({
    ...base,
    color: "var(--color-text)",
    margin: 0,
    padding: 0,
  }),
  singleValue: (base) => ({
    ...base,
    color: "var(--color-text)",
    fontWeight: 600,
  }),
  dropdownIndicator: (base, state) => ({
    ...base,
    padding: 4,
    color: state.isFocused
      ? "var(--color-accent)"
      : "color-mix(in srgb, var(--color-muted) 80%, transparent)",
    ":hover": {
      color: "var(--color-accent)",
    },
  }),
  clearIndicator: (base) => ({
    ...base,
    padding: 4,
    color: "color-mix(in srgb, var(--color-muted) 80%, transparent)",
    ":hover": {
      color: "var(--color-accent)",
    },
  }),
  menu: (provided) => ({
    ...provided,
    zIndex: 30,
    backgroundColor: glassBg,
    backdropFilter: "blur(var(--glass-blur))",
    color: "var(--color-text)",
    border: `1px solid ${glassBorder}`,
    borderRadius: 8,
    boxShadow: "var(--shadow-glass-hover)",
  }),
  menuList: (provided) => ({
    ...provided,
    backgroundColor: "transparent",
  }),
  option: (base, state) => ({
    ...base,
    backgroundColor: state.isSelected
      ? focusBackground
      : state.isFocused
        ? hoverBackground
        : "transparent",
    color: "var(--color-text)",
    cursor: state.isDisabled ? "not-allowed" : base.cursor,
    opacity: state.isDisabled ? 0.5 : 1,
  }),
  placeholder: (base) => ({
    ...base,
    color: "color-mix(in srgb, var(--color-muted) 65%, transparent)",
  }),
};

export const Select: React.FC<SelectProps> = React.memo(
  ({
    value,
    options,
    placeholder,
    disabled,
    isLoading,
    isClearable = true,
    onChange,
    onBlur,
    className = "",
    isCreatable,
    formatCreateLabel,
    onCreateOption,
  }) => {
    const selectValue = React.useMemo(() => {
      if (!value) return null;
      const existing = options.find((option) => option.value === value);
      if (existing) return existing;
      return { value, label: value, isDisabled: false };
    }, [value, options]);

    const handleChange = (
      option: SingleValue<SelectOption>,
      action: ActionMeta<SelectOption>,
    ) => {
      onChange(option?.value ?? null, action);
    };

    const sharedProps: Partial<ReactSelectProps<SelectOption, false>> = {
      className,
      classNamePrefix: "app-select",
      value: selectValue,
      options,
      onChange: handleChange,
      placeholder,
      isDisabled: disabled,
      isLoading,
      onBlur,
      isClearable,
      styles: selectStyles,
    };

    if (isCreatable) {
      return (
        <CreatableSelect<SelectOption, false>
          {...sharedProps}
          onCreateOption={onCreateOption}
          formatCreateLabel={formatCreateLabel}
        />
      );
    }

    return <SelectComponent<SelectOption, false> {...sharedProps} />;
  },
);

Select.displayName = "Select";
