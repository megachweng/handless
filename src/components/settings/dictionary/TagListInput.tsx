import React, { useState } from "react";
import { toast } from "sonner";
import { X } from "@phosphor-icons/react";
import { useSettings } from "@/hooks/useSettings";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { SettingContainer } from "@/components/ui/SettingContainer";

interface TagListInputProps {
  settingKey: "dictionary_terms" | "custom_words";
  title: string;
  description: string;
  placeholder: string;
  addLabel: string;
  removeAriaLabel: (item: string) => string;
  duplicateMessage: (item: string) => string;
  maxLength?: number;
  allowSpaces?: boolean;
  sanitize?: (value: string) => string;
  inputClassName?: string;
}

export const TagListInput: React.FC<TagListInputProps> = ({
  settingKey,
  title,
  description,
  placeholder,
  addLabel,
  removeAriaLabel,
  duplicateMessage,
  maxLength = 100,
  allowSpaces = true,
  sanitize,
  inputClassName = "max-w-48",
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const [newItem, setNewItem] = useState("");
  const items = (getSetting(settingKey) as string[] | undefined) ?? [];

  const handleAdd = () => {
    let value = newItem.trim();
    if (sanitize) value = sanitize(value);
    if (!value || value.length > maxLength) return;
    if (!allowSpaces && value.includes(" ")) return;

    if (items.includes(value)) {
      toast.error(duplicateMessage(value));
      return;
    }
    updateSetting(settingKey, [...items, value]);
    setNewItem("");
  };

  const handleRemove = (itemToRemove: string) => {
    updateSetting(
      settingKey,
      items.filter((item) => item !== itemToRemove),
    );
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  };

  const trimmed = newItem.trim();
  const isAddDisabled =
    !trimmed ||
    trimmed.length > maxLength ||
    (!allowSpaces && newItem.includes(" ")) ||
    isUpdating(settingKey);

  return (
    <>
      <SettingContainer
        title={title}
        description={description}
        descriptionMode="tooltip"
        grouped
      >
        <div className="flex items-center gap-2">
          <Input
            type="text"
            className={inputClassName}
            value={newItem}
            onChange={(e) => setNewItem(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            variant="compact"
            disabled={isUpdating(settingKey)}
          />
          <Button
            onClick={handleAdd}
            disabled={isAddDisabled}
            variant="default"
            size="default"
          >
            {addLabel}
          </Button>
        </div>
      </SettingContainer>
      {items.length > 0 && (
        <div className="px-3 py-1.5 flex flex-wrap gap-1">
          {items.map((item) => (
            <Button
              key={item}
              onClick={() => handleRemove(item)}
              disabled={isUpdating(settingKey)}
              variant="secondary"
              size="sm"
              className="inline-flex items-center gap-1 cursor-pointer"
              aria-label={removeAriaLabel(item)}
            >
              <span>{item}</span>
              <X className="w-3 h-3" />
            </Button>
          ))}
        </div>
      )}
    </>
  );
};
