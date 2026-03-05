import React, { useState, useRef, useEffect, useMemo, useCallback, useLayoutEffect } from "react";
import { createPortal } from "react-dom";
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
  searchable?: boolean;
  creatable?: boolean;
  onCreateValue?: (value: string) => void;
  formatCreateLabel?: (input: string) => string;
  searchPlaceholder?: string;
}

const ChevronIcon: React.FC<{ rotated?: boolean }> = ({ rotated }) => (
  <svg
    className={cn(
      "w-4 h-4 ms-2 transition-transform duration-200",
      rotated && "rotate-180",
    )}
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
);

/**
 * Searchable / creatable variant using a popover-style dropdown with a text
 * input for filtering.  Falls back to the simple Radix Select when neither
 * `searchable` nor `creatable` is set.
 */
const SearchableDropdown: React.FC<DropdownProps> = ({
  options,
  selectedValue,
  onSelect,
  className = "",
  placeholder = "Select an option...",
  disabled = false,
  creatable = false,
  onCreateValue,
  formatCreateLabel,
  searchPlaceholder,
}) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Position the portalled popover below the trigger, updating on scroll/resize
  useLayoutEffect(() => {
    if (!isOpen || !triggerRef.current || !popoverRef.current) return;
    const update = () => {
      if (!triggerRef.current || !popoverRef.current) return;
      const rect = triggerRef.current.getBoundingClientRect();
      Object.assign(popoverRef.current.style, {
        position: "fixed",
        top: `${rect.bottom + 4}px`,
        left: `${rect.left}px`,
        width: `${rect.width}px`,
      });
    };
    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Node;
      if (
        triggerRef.current?.contains(target) ||
        popoverRef.current?.contains(target)
      ) {
        return;
      }
      setIsOpen(false);
      setSearch("");
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  const filteredOptions = useMemo(() => {
    if (!search) return options;
    const q = search.toLowerCase();
    return options.filter((opt) => opt.label.toLowerCase().includes(q));
  }, [options, search]);

  const showCreateOption = useMemo(() => {
    if (!creatable) return false;
    const trimmed = search.trim();
    if (trimmed === "") return false;
    const lower = trimmed.toLowerCase();
    return !options.some(
      (opt) => opt.label.toLowerCase() === lower || opt.value === trimmed,
    );
  }, [creatable, search, options]);

  // Total selectable items: filtered options + optional create option
  const totalItems = filteredOptions.length + (showCreateOption ? 1 : 0);

  // Reset highlight when the list changes
  useEffect(() => {
    setHighlightedIndex(totalItems > 0 ? 0 : -1);
  }, [totalItems]);

  const selectedLabel = useMemo(() => {
    const match = options.find((opt) => opt.value === selectedValue);
    if (match) return match.label;
    // For creatable: show the raw value when it's not in the options list
    if (selectedValue) return selectedValue;
    return undefined;
  }, [options, selectedValue]);

  const handleSelect = useCallback(
    (value: string) => {
      onSelect(value);
      setIsOpen(false);
      setSearch("");
    },
    [onSelect],
  );

  const handleCreate = useCallback(() => {
    const trimmed = search.trim();
    if (!trimmed) return;
    if (onCreateValue) {
      onCreateValue(trimmed);
    } else {
      onSelect(trimmed);
    }
    setIsOpen(false);
    setSearch("");
  }, [search, onCreateValue, onSelect]);

  // Scroll highlighted item into view
  const scrollHighlightedIntoView = useCallback((index: number) => {
    if (!listRef.current) return;
    const items = listRef.current.querySelectorAll("[data-option-index]");
    items[index]?.scrollIntoView({ block: "nearest" });
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setIsOpen(false);
      setSearch("");
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setHighlightedIndex((prev) => {
        const next = prev < totalItems - 1 ? prev + 1 : 0;
        scrollHighlightedIntoView(next);
        return next;
      });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setHighlightedIndex((prev) => {
        const next = prev > 0 ? prev - 1 : totalItems - 1;
        scrollHighlightedIntoView(next);
        return next;
      });
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (highlightedIndex >= 0 && highlightedIndex < filteredOptions.length) {
        handleSelect(filteredOptions[highlightedIndex].value);
      } else if (showCreateOption && highlightedIndex === filteredOptions.length) {
        handleCreate();
      }
    }
  };

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={cn(
          "px-2 py-1 text-sm font-semibold bg-glass-bg border border-glass-border rounded backdrop-blur-sm",
          "min-w-[160px] text-start flex items-center justify-between",
          "transition-all duration-150",
          disabled
            ? "opacity-50 cursor-not-allowed"
            : "hover:bg-glass-highlight cursor-pointer hover:border-glass-border-hover",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
          className,
        )}
      >
        <span className="truncate">
          {selectedLabel || placeholder}
        </span>
        <ChevronIcon rotated={isOpen} />
      </button>

      {isOpen &&
        createPortal(
          <div
            ref={popoverRef}
            style={{ position: "fixed" }}
            className={cn(
              "z-50 rounded-lg shadow-glass-hover overflow-hidden",
              "border border-glass-border",
              "bg-[var(--color-glass-bg-solid)]",
            )}
          >
            <div className="p-2 border-b border-glass-border">
              <input
                ref={searchInputRef}
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={searchPlaceholder || t("common.search")}
                className="w-full px-2 py-1 text-sm bg-glass-bg border border-glass-border rounded focus:outline-none focus:ring-1 focus:ring-accent focus:border-accent"
              />
            </div>
            <div ref={listRef} className="max-h-48 overflow-y-auto">
              {filteredOptions.length === 0 && !showCreateOption ? (
                <div className="px-2 py-2 text-sm text-muted-foreground text-center">
                  {t("common.noOptionsFound")}
                </div>
              ) : (
                <>
                  {filteredOptions.map((option, index) => (
                    <button
                      key={option.value}
                      type="button"
                      disabled={option.disabled}
                      data-option-index={index}
                      className={cn(
                        "w-full px-2 py-1 text-sm text-start transition-colors duration-150 cursor-pointer",
                        selectedValue === option.value
                          ? "bg-primary/20 font-semibold"
                          : highlightedIndex === index
                            ? "bg-primary/10"
                            : "hover:bg-primary/10",
                        option.disabled && "opacity-50 cursor-not-allowed",
                      )}
                      onMouseEnter={() => setHighlightedIndex(index)}
                      onClick={() => handleSelect(option.value)}
                    >
                      {option.label}
                    </button>
                  ))}
                  {showCreateOption && (
                    <button
                      type="button"
                      data-option-index={filteredOptions.length}
                      className={cn(
                        "w-full px-2 py-1 text-sm text-start transition-colors duration-150 cursor-pointer text-accent",
                        highlightedIndex === filteredOptions.length
                          ? "bg-primary/10"
                          : "hover:bg-primary/10",
                      )}
                      onMouseEnter={() => setHighlightedIndex(filteredOptions.length)}
                      onClick={handleCreate}
                    >
                      {formatCreateLabel
                        ? formatCreateLabel(search.trim())
                        : `Use "${search.trim()}"`}
                    </button>
                  )}
                </>
              )}
            </div>
          </div>,
          document.body,
        )}
    </>
  );
};

/** Simple Radix Select – used when `searchable` and `creatable` are both falsy. */
const SimpleDropdown: React.FC<DropdownProps> = ({
  options,
  selectedValue,
  onSelect,
  className = "",
  placeholder = "Select an option...",
  disabled = false,
}) => {
  const { t } = useTranslation();

  return (
    <SelectPrimitive.Root
      value={selectedValue ?? undefined}
      onValueChange={onSelect}
      disabled={disabled}
    >
      <SelectPrimitive.Trigger
        className={cn(
          "px-2 py-1 text-sm font-semibold bg-glass-bg border border-glass-border rounded backdrop-blur-sm",
          "min-w-[160px] text-start flex items-center justify-between",
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
          <ChevronIcon />
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

export const Dropdown: React.FC<DropdownProps> = (props) => {
  if (props.searchable || props.creatable) {
    return <SearchableDropdown {...props} />;
  }
  return <SimpleDropdown {...props} />;
};
